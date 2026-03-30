//! Whisper-based transcription runtime

use crate::protocol::transcript::{TranscriptProcessor, TranscriptSegment};
use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor};
use candle_transformers::models::whisper::{model::Whisper, Config};
use hf_hub::{api::sync::Api, Repo, RepoType};
use std::path::Path;
use tokenizers::Tokenizer;

pub struct WhisperTranscriptProcessor;

impl WhisperTranscriptProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WhisperTranscriptProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptProcessor for WhisperTranscriptProcessor {
    fn transcribe(&self, path: &Path) -> Result<Vec<TranscriptSegment>> {
        let device = Device::Cpu;

        let api = Api::new().context("failed to create hf-hub api")?;
        let repo = api.repo(Repo::new(
            "openai/whisper-tiny".to_string(),
            RepoType::Model,
        ));

        let config_filename = repo
            .get("config.json")
            .context("failed to get config.json")?;
        let tokenizer_filename = repo
            .get("tokenizer.json")
            .context("failed to get tokenizer.json")?;
        let weights_filename = repo
            .get("model.safetensors")
            .context("failed to get model.safetensors")?;

        let config: Config = serde_json::from_str(&std::fs::read_to_string(config_filename)?)
            .context("failed to parse config")?;
        let tokenizer = Tokenizer::from_file(tokenizer_filename)
            .map_err(anyhow::Error::msg)
            .context("failed to load tokenizer")?;

        // Debug: Check if tokenizer has required special tokens
        tracing::debug!("Tokenizer vocab size: {}", tokenizer.get_vocab_size(true));
        let has_sot = tokenizer.token_to_id("<|startoftranscript|>").is_some();
        let has_eot = tokenizer.token_to_id("<|endoftranscript|>").is_some();
        let has_transcribe = tokenizer.token_to_id("<|transcribe|>").is_some();
        tracing::debug!(
            "Special tokens present: SOT={}, EOT={}, Transcribe={}",
            has_sot,
            has_eot,
            has_transcribe
        );

        if !has_sot || !has_eot || !has_transcribe {
            tracing::warn!(
                "Tokenizer missing special tokens. This may indicate wrong tokenizer version."
            );
            tracing::warn!("Expected Whisper tokenizer with: <|startoftranscript|>, <|endoftranscript|>, <|transcribe|>");
        }

        if let Some(sot) = tokenizer.token_to_id("<|startoftranscript|>") {
            tracing::debug!("SOT token ID: {}", sot);
        } else {
            tracing::warn!("SOT token not found in tokenizer");
        }
        if let Some(eot) = tokenizer.token_to_id("<|endoftranscript|>") {
            tracing::debug!("EOT token ID: {}", eot);
        } else {
            tracing::warn!("EOT token not found in tokenizer");
        }

        let vb = unsafe {
            candle_nn::VarBuilder::from_mmaped_safetensors(
                &[weights_filename],
                DType::F32,
                &device,
            )?
        };

        let mut model = Whisper::load(&vb, config.clone()).context("failed to load model")?;

        let audio_data = load_audio_as_f32(path)?;
        tracing::debug!("Loaded {} audio samples", audio_data.len());

        let mel = pcm_to_mel(&config, &audio_data, &device)?;
        let mel_dims = mel.dims();
        tracing::debug!("Mel spectrogram dimensions: {:?}", mel_dims);
        let mel_len = mel_dims[2];

        if mel_len == 0 {
            tracing::warn!("Empty mel spectrogram, returning no speech detected");
            return Ok(vec![TranscriptSegment {
                start: 0.0,
                end: 30.0,
                text: "[No speech detected]".to_string(),
                confidence: 0.0,
            }]);
        }

        let segments = decode_greedy(&mut model, &tokenizer, &mel, &config, mel_len)?;

        Ok(segments)
    }

    fn to_srt(&self, transcript: &[TranscriptSegment], output: &Path) -> Result<()> {
        crate::exporter::srt::export_srt(transcript, output)
    }

    fn to_chapters(&self, transcript: &[TranscriptSegment], output: &Path) -> Result<()> {
        crate::exporter::chapters::export_youtube_chapters(transcript, output)
    }

    fn to_ass(&self, transcript: &[TranscriptSegment], output: &Path) -> Result<()> {
        crate::exporter::ass::export_ass(transcript, output)
    }
}

fn load_audio_as_f32(path: &Path) -> Result<Vec<f32>> {
    let output = std::process::Command::new("ffmpeg")
        .args([
            "-i",
            path.to_str().context("invalid path")?,
            "-ar",
            "16000",
            "-ac",
            "1",
            "-f",
            "f32le",
            "-",
        ])
        .output()
        .context("failed to extract audio with ffmpeg")?;

    let bytes = output.stdout;
    let samples: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    Ok(samples)
}

fn pcm_to_mel(config: &Config, pcm: &[f32], device: &Device) -> Result<Tensor> {
    let n_fft = 400;
    let hop_length = 160;
    let n_mels = config.num_mel_bins;

    let padded_len = pcm.len() + n_fft / 2 * 2;
    let mut padded = vec![0.0f32; padded_len];
    padded[n_fft / 2..n_fft / 2 + pcm.len()].copy_from_slice(pcm);

    let n_frames = (padded_len - n_fft) / hop_length + 1;
    let mut mel_spec = vec![0.0f32; n_mels * n_frames];

    for frame in 0..n_frames {
        let start = frame * hop_length;

        for mel_bin in 0..n_mels {
            let freq_low = mel_bin * 8000 / n_mels;
            let freq_high = (mel_bin + 1) * 8000 / n_mels;
            let bin_low = freq_low * n_fft / 16000;
            let bin_high = (freq_high * n_fft / 16000).min(n_fft);

            let mut energy = 0.0f32;
            for i in bin_low..bin_high {
                if start + i < padded.len() {
                    let window =
                        0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / n_fft as f32).cos());
                    energy += padded[start + i] * window;
                }
            }
            mel_spec[mel_bin * n_frames + frame] = energy.abs().ln_1p();
        }
    }

    Tensor::from_vec(mel_spec, (1, n_mels, n_frames), device).map_err(anyhow::Error::msg)
}

fn decode_greedy(
    model: &mut Whisper,
    tokenizer: &Tokenizer,
    mel: &Tensor,
    config: &Config,
    mel_len: usize,
) -> Result<Vec<TranscriptSegment>> {
    let sot_token = tokenizer
        .token_to_id("<|startoftranscript|>")
        .context("missing sot token - tokenizer may be corrupted or wrong version")?;
    // EOT token may be missing in some tokenizer versions, use hardcoded fallback
    let eot_token = tokenizer
        .token_to_id("<|endoftranscript|>")
        .unwrap_or(50257); // Standard Whisper EOT token ID
    let transcribe_token = tokenizer
        .token_to_id("<|transcribe|>")
        .context("missing transcribe token - tokenizer may be corrupted or wrong version")?;
    let no_speech_token = tokenizer.token_to_id("<|nospeech|>").unwrap_or(eot_token);

    let chunk_size = 3000;
    let mut segments = Vec::new();

    for chunk_start in (0..mel_len).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(mel_len);
        let chunk_len = chunk_end - chunk_start;
        if chunk_len < 100 {
            continue;
        }

        let chunk_mel = mel.narrow(2, chunk_start, chunk_len).context(format!(
            "failed to narrow mel tensor: start={}, len={}, mel_dims={:?}",
            chunk_start,
            chunk_len,
            mel.dims()
        ))?;
        let chunk_encoder_output = model.encoder.forward(&chunk_mel, true).context(format!(
            "failed to run encoder: chunk_mel_dims={:?}",
            chunk_mel.dims()
        ))?;

        let mut tokens = vec![sot_token, transcribe_token];
        let mut token_probs = Vec::new();

        for _ in 0..config.max_target_positions.min(448) {
            let input = Tensor::new(tokens.clone(), mel.device())?.unsqueeze(0)?;

            let logits = model.decoder.forward(&input, &chunk_encoder_output, true)?;
            let seq_len = logits.dims()[1];
            let next_token_logits = logits.get(seq_len - 1)?;

            let next_token = next_token_logits.argmax(0)?.to_scalar::<u32>()?;

            if next_token == eot_token || next_token == no_speech_token {
                break;
            }

            let probs = candle_nn::ops::softmax(&next_token_logits, 0)?;
            let prob = probs.get(next_token as usize)?.to_scalar::<f32>()?;
            token_probs.push(prob);

            tokens.push(next_token);

            if tokens.len() > 400 {
                break;
            }
        }

        let text_tokens: Vec<u32> = tokens[2..].to_vec();
        if text_tokens.is_empty() {
            continue;
        }

        let text = tokenizer
            .decode(&text_tokens, true)
            .map_err(anyhow::Error::msg)?;

        if text.is_empty() || text.trim().is_empty() {
            continue;
        }

        let time_start = chunk_start as f32 / 100.0;
        let time_end = chunk_end as f32 / 100.0;

        let confidence = if token_probs.is_empty() {
            0.5
        } else {
            token_probs.iter().sum::<f32>() / token_probs.len() as f32
        };

        segments.push(TranscriptSegment {
            start: time_start,
            end: time_end,
            text: text.trim().to_string(),
            confidence,
        });
    }

    if segments.is_empty() {
        segments.push(TranscriptSegment {
            start: 0.0,
            end: 30.0,
            text: "[No speech detected]".to_string(),
            confidence: 0.0,
        });
    }

    Ok(segments)
}
