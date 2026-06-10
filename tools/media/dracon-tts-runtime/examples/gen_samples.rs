use dracon_tts_runtime::kokoro::KokoroTts;
use std::fs;

#[tokio::main]
async fn main() {
    fs::create_dir_all("/home/dracon/Dev/Remi/test_extended_dsp").ok();
    fs::create_dir_all("/home/dracon/Dev/Remi/test_extended_nodsp").ok();

    let long_text = "The quick brown fox jumps over the lazy dog. This is a test of the Kokoro TTS system. We are generating samples to test volume consistency across multiple runs.";

    let model_path = "/home/dracon/Dev/Remi/assets/models/kokoro-v1.0.onnx";
    let voices_dir = "/home/dracon/Dev/Remi/assets/models";

    // Just use speak_nowait to play audio and dump chunks
    println!("Generating DSP samples (playing audio)...");
    // SAFETY: example-only env var mutation
    unsafe {
        std::env::set_var(
            "REMI_TTS_DUMP_CHUNKS_DIR",
            "/home/dracon/Dev/Remi/test_extended_dsp",
        );
        std::env::remove_var("REMI_KOKORO_NO_DSP");
    }

    for i in 1..=10 {
        println!("DSP sample {}...", i);
        let tts = KokoroTts::new_with_voice(model_path, voices_dir, "af_skye")
            .await
            .expect("load Kokoro TTS");
        let text = format!("{} [test {}]", long_text, i);
        tts.speak_nowait(&text).await;
        tts.wait_until_done().await;
    }

    println!("\nGenerating NoDSP samples (playing audio)...");
    // SAFETY: example-only env var mutation
    unsafe {
        std::env::set_var(
            "REMI_TTS_DUMP_CHUNKS_DIR",
            "/home/dracon/Dev/Remi/test_extended_nodsp",
        );
        std::env::set_var("REMI_KOKORO_NO_DSP", "1");
    }

    for i in 1..=10 {
        println!("NoDSP sample {}...", i);
        let tts = KokoroTts::new_with_voice(model_path, voices_dir, "af_skye")
            .await
            .expect("load Kokoro TTS");
        let text = format!("{} [test {}]", long_text, i);
        tts.speak_nowait(&text).await;
        tts.wait_until_done().await;
    }

    println!("\nDone!");
}
