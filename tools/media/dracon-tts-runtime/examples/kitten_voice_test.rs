use dracon_tts_runtime::kitten::{KittenTTS, VOICE_DESCRIPTIONS};
use std::fs;

#[tokio::main]
async fn main() {
    fs::create_dir_all("/home/dracon/Dev/Remi/kitten_voice_tests").ok();

    let model_path = "/home/dracon/Dev/Remi/assets/models/kitten_tts_nano_v0_8.onnx";
    let voices_path = "/home/dracon/Dev/Remi/assets/models/voices_v0_8.npz";

    // Longer test sentences
    let sentences = [
        "The ancient library stood at the edge of the village, its stone walls covered in centuries of moss and ivy. Inside, dust motes danced in the beams of light that filtered through tall windows, illuminating shelves that reached toward the ceiling like fingers grasping for knowledge.",

        "Technology has fundamentally changed how we communicate, work, and live our daily lives. From smartphones that connect us to people across the globe, to artificial intelligence systems that can write poetry and solve complex mathematical problems, the pace of innovation continues to accelerate.",

        "In the heart of the mountain range, where the air grows thin and crisp, there exists a hidden valley that few have ever seen. The meadow there blooms with flowers found nowhere else on Earth, and a crystal-clear stream winds its way through the grass, singing an eternal song.",

        "The chef carefully selected each ingredient, knowing that the secret to a truly memorable dish lay not just in the recipe, but in the quality and freshness of every component. With practiced hands, she began to chop, slice, and prepare, transforming raw materials into culinary art.",

        "Mathematics is often called the language of the universe, and for good reason. From the spiral patterns of galaxies to the intricate structures of snowflakes, mathematical principles govern the world around us in ways both visible and hidden, waiting to be discovered by curious minds.",
    ];

    // Test each voice
    for (internal_name, friendly_name, gender) in VOICE_DESCRIPTIONS {
        println!(
            "\n=== Testing {} ({}) - {} ===",
            friendly_name, internal_name, gender
        );

        for (i, text) in sentences.iter().enumerate() {
            println!("Generating sentence {} for {}...", i + 1, friendly_name);

            let tts = KittenTTS::new_with_voice(model_path, voices_path, internal_name)
                .await
                .expect("load Kitten TTS");

            // Use speak_nowait to generate audio
            tts.speak_nowait(text).await;
            tts.wait_until_done().await;

            println!("Done: {} - sentence {}", friendly_name, i + 1);
        }

        // Small delay between voices
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    println!("\n\nAll voice tests complete!");
    println!("Voices tested:");
    for (internal, friendly, gender) in VOICE_DESCRIPTIONS {
        println!("  - {} ({}) [{}]", friendly, internal, gender);
    }
}
