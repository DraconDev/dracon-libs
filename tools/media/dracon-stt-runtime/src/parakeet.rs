use crate::stt_contracts::{
    EngineCapabilities, SpeechToText, TimestampedTranscription, TranscriptionResult,
};
use parakeet_rs::{Parakeet, TimestampMode, Transcriber};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Parakeet speech-to-text backend.
pub struct ParakeetStt {
    model: Arc<Mutex<Parakeet>>,
    ready: bool,
}

impl ParakeetStt {
    /// Create a Parakeet backend from a model directory or bundled assets path.
    pub fn new(model_dir: &str) -> anyhow::Result<Self> {
        let model_path = Path::new(model_dir);
        let model = if model_path.exists() {
            let onnx_path = model_path.join("onnx");
            if onnx_path.exists() {
                Parakeet::from_pretrained(&onnx_path, None).map_err(|e| {
                    anyhow::anyhow!("Failed to load Parakeet model from {:?}: {}", onnx_path, e)
                })?
            } else {
                Parakeet::from_pretrained(model_path, None).map_err(|e| {
                    anyhow::anyhow!("Failed to load Parakeet model from {:?}: {}", model_path, e)
                })?
            }
        } else {
            let fallback_path = Path::new("assets/models/parakeet-ctc/onnx");
            if fallback_path.exists() {
                Parakeet::from_pretrained(fallback_path, None).map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to load Parakeet model from {:?}: {}",
                        fallback_path,
                        e
                    )
                })?
            } else {
                return Err(anyhow::anyhow!(
                    "Parakeet model not found. Download from: https://huggingface.co/onnx-community/parakeet-ctc-0.6b-ONNX"
                ));
            }
        };

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            ready: true,
        })
    }

    /// Transcribe audio samples asynchronously and return text when available.
    pub async fn transcribe_audio(&self, audio_data: Vec<f32>) -> Option<String> {
        println!("STT: Transcribing {} samples...", audio_data.len());

        if audio_data.len() < 16000 {
            println!("STT: Audio too short, skipping");
            return Ok(None);
        }

        let rms: f32 =
            (audio_data.iter().map(|x| x * x).sum::<f32>() / audio_data.len() as f32).sqrt();
        println!("STT: Audio RMS energy: {:.6}", rms);

        if rms < 0.001 {
            println!("STT: Audio appears to be silence (RMS too low), skipping");
            return Ok(None);
        }

        let model = self.model.clone();

        let result = tokio::task::spawn_blocking(move || {
            let start = std::time::Instant::now();
            let mut model = model.blocking_lock();
            let result =
                model.transcribe_samples(audio_data, 16000, 1, Some(TimestampMode::Sentences));
            let elapsed = start.elapsed();
            println!("STT: Parakeet inference: {:.2}s", elapsed.as_secs_f32());
            result
        })
        .await;

        match result {
            Ok(Ok(transcription)) => {
                let text = transcription.text.trim().to_string();
                if text.is_empty() {
                    println!("STT: Transcription returned empty string");
                    Ok(None)
                } else {
                    println!("STT: Transcribed: \"{}\"", text);
                    Ok(Some(text))
                }
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("STT Error: {e}")),
            Err(e) => Err(anyhow::anyhow!("STT Task Error: {e:?}")),
        }
    }
}

impl SpeechToText for ParakeetStt {
    fn transcribe(
        &self,
        audio: &[f32],
        sample_rate: u32,
    ) -> crate::stt_contracts::SttResult<Option<TranscriptionResult>> {
        if audio.len() < 16000 {
            return Ok(None);
        }

        let rms: f32 = (audio.iter().map(|x| x * x).sum::<f32>() / audio.len() as f32).sqrt();
        if rms < 0.001 {
            return Ok(None);
        }

        let model = self.model.clone();
        let audio = audio.to_vec();

        let result = std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let mut model = model.blocking_lock();
            let result =
                model.transcribe_samples(audio, sample_rate, 1, Some(TimestampMode::Sentences));
            let elapsed = start.elapsed();
            println!("STT: Parakeet inference: {:.2}s", elapsed.as_secs_f32());
            result
        })
        .join();

        match result {
            Ok(Ok(transcription)) => {
                let text = transcription.text.trim().to_string();
                if text.is_empty() {
                    Ok(None)
                } else {
                    println!("STT: Transcribed: \"{}\"", text);
                    Ok(Some(TranscriptionResult::new(text)))
                }
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("STT Error: {}", e)),
            Err(_) => Err(anyhow::anyhow!("STT Task Error")),
        }
    }

    fn name(&self) -> &'static str {
        "Parakeet"
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
            supports_streaming: true,
            supports_language_detection: false,
        }
    }
}

impl TimestampedTranscription for ParakeetStt {
    fn transcribe_with_timestamps(
        &self,
        audio: &[f32],
        sample_rate: u32,
    ) -> SttResult<Option<Vec<TimestampedSegment>>> {
        if audio.len() < 16000 {
            return Ok(None);
        }

        let rms: f32 = (audio.iter().map(|x| x * x).sum::<f32>() / audio.len() as f32).sqrt();
        if rms < 0.001 {
            return Ok(None);
        }

        let model = self.model.clone();
        let audio = audio.to_vec();
        let result = std::thread::spawn(move || {
            let mut model = model.blocking_lock();
            model.transcribe_samples(audio, sample_rate, 1, Some(TimestampMode::Sentences))
        })
        .join();

        match result {
            Ok(Ok(transcription)) => {
                let segments = transcription
                    .tokens
                    .iter()
                    .map(|token| TimestampedSegment {
                        start_secs: f64::from(token.start),
                        end_secs: f64::from(token.end),
                        text: token.text.clone(),
                        confidence: 0.0,
                    })
                    .collect();
                Ok(Some(segments))
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("STT Error: {e}")),
            Err(_) => Err(anyhow::anyhow!("STT Task Error")),
        }
    }
}

// SAFETY: ParakeetStt is Send because the Parakeet model is wrapped in Arc<Mutex<Parakeet>>,
// which provides shared ownership and interior mutability. All access is gated by the Mutex,
// ensuring synchronized access from any thread.
//
// SAFETY: ParakeetStt is Sync because Parakeet operations are behind Arc<Mutex<Parakeet>>.
// The Mutex ensures only one thread accesses the model at a time, and the model itself
// is thread-safe for concurrent reads via its internal synchronization.
unsafe impl Send for ParakeetStt {}
unsafe impl Sync for ParakeetStt {}
