pub mod contracts;
pub mod kitten;
pub mod kokoro;

pub use contracts::{
    DynTtsEngine, Gender, SynthesisRequest, SynthesisResult, TextToSpeech, TtsConfig,
    TtsEngineBase, TtsResult, VoiceInfo, VoiceProvider,
};

pub use kitten::KittenTTS;
pub use kokoro::KokoroTts;

#[derive(Clone)]
pub enum TtsEngine {
    Kitten(std::sync::Arc<KittenTTS>),
    Kokoro(std::sync::Arc<KokoroTts>),
}

impl TtsEngine {
    pub fn speak(&self, text: &str) {
        match self {
            TtsEngine::Kitten(k) => TextToSpeech::speak(&**k, text),
            TtsEngine::Kokoro(k) => TextToSpeech::speak(&**k, text),
        }
    }

    pub fn stop(&self) {
        match self {
            TtsEngine::Kitten(k) => TextToSpeech::stop(&**k),
            TtsEngine::Kokoro(k) => TextToSpeech::stop(&**k),
        }
    }

    pub fn is_speaking(&self) -> bool {
        match self {
            TtsEngine::Kitten(k) => TextToSpeech::is_speaking(&**k),
            TtsEngine::Kokoro(k) => TextToSpeech::is_speaking(&**k),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            TtsEngine::Kitten(k) => TextToSpeech::name(&**k),
            TtsEngine::Kokoro(k) => TextToSpeech::name(&**k),
        }
    }

    pub async fn speak_nowait(&self, text: &str) {
        match self {
            TtsEngine::Kitten(k) => k.speak_nowait(text).await,
            TtsEngine::Kokoro(k) => k.speak_nowait(text).await,
        }
    }

    pub async fn wait_until_done(&self) {
        match self {
            TtsEngine::Kitten(k) => k.wait_until_done().await,
            TtsEngine::Kokoro(k) => k.wait_until_done().await,
        }
    }
}
