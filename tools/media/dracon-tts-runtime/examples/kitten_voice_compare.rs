use dracon_tts_runtime::kitten::{KittenTTS, VOICE_DESCRIPTIONS};
use std::fs;

#[tokio::main]
async fn main() {
    let output_dir = "/home/dracon/Dev/Remi/kitten_voice_tests";
    fs::create_dir_all(output_dir).ok();

    // Model configs
    let models = vec![
        (
            "nano",
            "/home/dracon/Dev/Remi/assets/models/kitten_tts_nano_v0_8.onnx",
        ),
        (
            "micro",
            "/home/dracon/Dev/Remi/assets/models/kitten_micro.onnx",
        ),
    ];

    let voices_path = "/home/dracon/Dev/Remi/assets/models/voices_v0_8.npz";

    // Test sentences - varied lengths
    let sentences = vec![
        ("short", "Hello, how are you today?"),
        ("medium", "The weather is lovely this morning. I think we should go for a walk in the park and enjoy the sunshine while it lasts."),
        ("long1", "In the depths of the ancient forest, where sunlight barely penetrates the thick canopy of leaves, there exists a world unlike any other. The trees here have stood for thousands of years, their gnarled roots weaving intricate patterns through the soft earth."),
        ("long2", "Artificial intelligence has transformed nearly every aspect of modern computing. From natural language processing that enables machines to understand and generate human text, to computer vision systems that can identify objects and faces with superhuman accuracy."),
        ("long3", "I remember the first time I visited the old bookstore downtown. It was a rainy Tuesday afternoon, and I had wandered in seeking shelter from the sudden downpour. The smell of old paper and leather bindings instantly transported me to another time."),
    ];

    println!("Generating voice comparison files...");
    println!("Output directory: {}\n", output_dir);

    for (model_name, model_path) in &models {
        println!("=== Model: {} ===", model_name);

        for (internal_name, friendly_name, _gender) in VOICE_DESCRIPTIONS {
            for (sent_name, text) in &sentences {
                let filename = format!(
                    "{}/{}_{}_{}.wav",
                    output_dir,
                    model_name,
                    friendly_name.to_lowercase(),
                    sent_name
                );

                println!("  {} - {} - {}", model_name, friendly_name, sent_name);

                let tts = KittenTTS::new_with_voice(model_path, voices_path, internal_name).await.expect("load Kitten TTS");
                let samples = tts.synthesize(text).expect("Failed to synthesize");
                tts.save_wav(&samples, &filename).expect("Failed to save");
            }
        }
        println!();
    }

    println!("\nDone! Files saved to: {}", output_dir);
    println!("\nStructure:");
    for (model_name, _) in &models {
        println!("  {}_*.wav (8 voices x 5 sentences)", model_name);
    }
}
