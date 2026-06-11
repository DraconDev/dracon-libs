//! Whisper-based transcription using Candle
//!
//! This implementation provides timestamped transcription for video subtitle use cases.
//! It uses the OpenAI Whisper model via HuggingFace Hub.

use crate::stt_contracts::{
    EngineCapabilities, SpeechToText, SttResult, TimestampedSegment, TimestampedTranscription,
    TranscriptionResult,
};
use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor};
use candle_transformers::models::whisper::{model::Whisper, Config};
use hf_hub::{api::sync::Api, Repo, RepoType};
use std::sync::Arc;
use tokenizers::Tokenizer;

/// Whisper speech-to-text backend.
pub struct WhisperStt {
    model: Arc<Mutex<WhisperState>>,
    tokenizer: Tokenizer,
    config: Config,
    ready: bool,
}

struct WhisperState {
    model: Whisper,
}

impl WhisperStt {
    /// Create a Whisper backend using the default tiny model.
    pub fn new() -> Result<Self> {
        Self::from_model("openai/whisper-tiny")
    }

    /// Create a Whisper backend from a HuggingFace model id.
    pub fn from_model(model_id: &str) -> Result<Self> {
        let device = Device::Cpu;

        let api = Api::new().context("failed to create hf-hub api")?;
        let repo = api.repo(Repo::new(model_id.to_string(), RepoType::Model));

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

        // SAFETY: from_mmaped_safetensors reads binary weight files and maps them into memory.
        // The mmap is read-only and dropped immediately after VarBuilder construction, so there
        // is no risk of aliased mutation. The caller guarantees weights_filename is a valid
        // safetensors file compatible with the model definition.
        let vb = unsafe {
            candle_nn::VarBuilder::from_mmaped_safetensors(
                &[weights_filename],
                DType::F32,
                &device,
            )?
        };

        let model = Whisper::load(&vb, config.clone()).context("failed to load model")?;

        println!("STT: Whisper model '{}' loaded successfully", model_id);

        Ok(Self {
            model: Arc::new(Mutex::new(WhisperState { model })),
            tokenizer,
            config,
            ready: true,
        })
    }

    /// Transcribe raw 16kHz PCM samples without timestamp segmentation.
    pub fn transcribe_raw(&self, audio: &[f32]) -> Result<Option<TranscriptionResult>> {
        if audio.len() < 16000 {
            return Ok(None);
        }

        let rms = (audio.iter().map(|x| x * x).sum::<f32>() / audio.len() as f32).sqrt();
        if rms < 0.001 {
            return Ok(None);
        }

        let mel = pcm_to_mel(&self.config, audio, &Device::Cpu)?;
        let mel_len = mel.dims()[2];

        let mut state = self
            .model
            .lock()
            .map_err(|_| anyhow::anyhow!("Whisper model mutex poisoned"))?;
        let segments = decode_greedy(&mut state.model, &self.tokenizer, &mel, mel_len)?;

        if segments.is_empty() {
            return Ok(None);
        }

        let combined_text = segments
            .iter()
            .map(|s| s.text.clone())
            .collect::<Vec<_>>()
            .join(" ");
        let avg_confidence =
            segments.iter().map(|s| s.confidence).sum::<f32>() / segments.len() as f32;

        Ok(Some(TranscriptionResult {
            text: combined_text,
            confidence: avg_confidence,
            language: None,
            duration_secs: segments.last().map(|s| s.end_secs as f32).unwrap_or(0.0),
        }))
    }
}

impl SpeechToText for WhisperStt {
    fn name(&self) -> &'static str {
        "Whisper"
    }

    fn sample_rate(&self) -> u32 {
        16000
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_timestamps: true,
            supports_streaming: false,
            supports_language_detection: true,
        }
    }

    fn transcribe(
        &self,
        audio: &[f32],
        sample_rate: u32,
    ) -> SttResult<Option<TranscriptionResult>> {
        if sample_rate != 16000 {
            anyhow::bail!("Whisper only supports 16kHz audio, got {}", sample_rate);
        }
        self.transcribe_raw(audio).map_err(anyhow::Error::msg)
    }
}

impl TimestampedTranscription for WhisperStt {
    fn transcribe_with_timestamps(
        &self,
        audio: &[f32],
        sample_rate: u32,
    ) -> SttResult<Option<Vec<TimestampedSegment>>> {
        if sample_rate != 16000 {
            anyhow::bail!("Whisper only supports 16kHz audio, got {}", sample_rate);
        }
        if audio.len() < 16000 {
            return Ok(None);
        }

        let rms = (audio.iter().map(|x| x * x).sum::<f32>() / audio.len() as f32).sqrt();
        if rms < 0.001 {
            return Ok(None);
        }

        let mel = pcm_to_mel(&self.config, audio, &Device::Cpu)?;
        let mel_len = mel.dims()[2];

        let mut state = self
            .model
            .lock()
            .map_err(|_| anyhow::anyhow!("Whisper model mutex poisoned"))?;
        let segments = decode_greedy(&mut state.model, &self.tokenizer, &mel, mel_len)?;

        if segments.is_empty() {
            return Ok(None);
        }

        Ok(Some(segments))
    }
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
    mel_len: usize,
) -> Result<Vec<TimestampedSegment>> {
    let sot_token = tokenizer
        .token_to_id("<|startoftranscript|>")
        .context("missing sot token")?;
    let eot_token = tokenizer
        .token_to_id("<|endoftranscript|>")
        .context("missing eot token")?;
    let transcribe_token = tokenizer
        .token_to_id("<|transcribe|>")
        .context("missing transcribe token")?;
    let no_speech_token = tokenizer.token_to_id("<|nospeech|>").unwrap_or(eot_token);

    let chunk_size = 3000;
    let mut segments = Vec::new();

    for chunk_start in (0..mel_len).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(mel_len);
        let chunk_len = chunk_end - chunk_start;
        if chunk_len < 100 {
            continue;
        }

        let chunk_mel = mel.narrow(2, chunk_start, chunk_len)?;
        let chunk_encoder_output = model.encoder.forward(&chunk_mel, true)?;

        let mut tokens = vec![sot_token, transcribe_token];
        let mut token_probs = Vec::new();

        for _ in 0..model.config.max_target_positions.min(448) {
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

        let time_start = chunk_start as f64 / 100.0;
        let time_end = chunk_end as f64 / 100.0;

        let confidence = if token_probs.is_empty() {
            0.5
        } else {
            token_probs.iter().sum::<f32>() / token_probs.len() as f32
        };

        segments.push(TimestampedSegment {
            start_secs: time_start,
            end_secs: time_end,
            text: text.trim().to_string(),
            confidence,
        });
    }

    if segments.is_empty() {
        segments.push(TimestampedSegment {
            start_secs: 0.0,
            end_secs: 30.0,
            text: "[No speech detected]".to_string(),
            confidence: 0.0,
        });
    }

    Ok(segments)
}

use std::sync::Mutex;
