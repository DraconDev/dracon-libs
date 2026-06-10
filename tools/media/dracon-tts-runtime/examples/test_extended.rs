use dracon_tts_runtime::kokoro::KokoroTts;
use std::fs;

#[tokio::main]
async fn main() {
    fs::create_dir_all("/home/dracon/Dev/Remi/test_extended_dsp").ok();
    fs::create_dir_all("/home/dracon/Dev/Remi/test_extended_nodsp").ok();

    let long_text = r#"The quick brown fox jumps over the lazy dog. This sentence contains every letter of the alphabet and has been used for typing practice for many generations. In the heart of the ancient forest, where sunlight filtered through the canopy in golden shafts, there lived a community of creatures who had learned to live in harmony. The birds sang their morning songs, the squirrels gathered nuts for the coming winter, and the deer grazed peacefully in the meadows. Technology has transformed our world in countless ways. From the invention of the wheel to the development of artificial intelligence, humanity has constantly pushed the boundaries of what is possible. Today, we stand at the threshold of a new era, where machines can learn, adapt, and even create. The ocean waves crashed against the rocky shore, each one leaving behind a trail of foam and seaweed. Seagulls circled overhead, their cries echoing across the beach. A lone figure walked along the water's edge, lost in thought, contemplating the vastness of the sea and the mysteries it holds. Mathematics is the language of the universe. From the simplest arithmetic to the most complex equations, numbers help us understand the world around us. Pi, the ratio of a circle's circumference to its diameter, extends infinitely without repeating, a testament to the infinite complexity of nature. As the sun set behind the mountains, painting the sky in shades of orange and purple, the city below began to light up. Street lamps flickered to life, windows glowed with warm light, and the hum of evening traffic filled the air. Another day was coming to an end, but the city never truly slept. Music has the power to move us in ways that words alone cannot. A simple melody can evoke memories long forgotten, stir emotions buried deep within, and connect people across cultures and generations. Whether it's the beat of a drum, the strum of a guitar, or the soaring notes of a violin, music speaks to the soul. In the quiet of the library, surrounded by shelves of books reaching toward the ceiling, a student sat at a wooden desk, poring over an ancient text. Each page turned revealed new knowledge, new ideas, new perspectives. The pursuit of wisdom is a journey without end. The garden was a riot of colors in spring. Roses bloomed in shades of red and pink, tulips stretched toward the sun, and lavender filled the air with its calming fragrance. Butterflies flitted from flower to flower, and bees buzzed busily, doing their important work of pollination. Cooking is both an art and a science. The right combination of ingredients, the perfect temperature, the precise timing, all these elements come together to create dishes that nourish not just the body but the spirit. A good meal shared with loved ones is one of life's greatest pleasures."#;

    let model_path = "/home/dracon/Dev/Remi/assets/models/kokoro-v1.0.onnx";
    let voices_dir = "/home/dracon/Dev/Remi/assets/models";

    println!("Testing Kokoro with DSP (10 long samples)...");
    std::env::remove_var("REMI_KOKORO_NO_DSP");

    for i in 1..=10 {
        println!("Generating DSP sample {}...", i);
        let tts = KokoroTts::new_with_voice(model_path, voices_dir, "af_skye")
            .await
            .expect("load Kokoro TTS");
        let samples = tts.synthesize(long_text).expect("Failed to synthesize");
        let path = format!(
            "/home/dracon/Dev/Remi/test_extended_dsp/sample_{:02}.wav",
            i
        );
        tts.save_wav(&samples, &path).expect("Failed to save");
        println!("Saved: {} ({} samples)", path, samples.len());
    }

    println!("\nTesting Kokoro without DSP (10 long samples)...");
    // SAFETY: example-only env var mutation
    unsafe {
        std::env::set_var("REMI_KOKORO_NO_DSP", "1");
    }

    for i in 1..=10 {
        println!("Generating NoDSP sample {}...", i);
        let tts = KokoroTts::new_with_voice(model_path, voices_dir, "af_skye")
            .await
            .expect("load Kokoro TTS");
        let samples = tts.synthesize(long_text).expect("Failed to synthesize");
        let path = format!(
            "/home/dracon/Dev/Remi/test_extended_nodsp/sample_{:02}.wav",
            i
        );
        tts.save_wav(&samples, &path).expect("Failed to save");
        println!("Saved: {} ({} samples)", path, samples.len());
    }

    println!("\nDone! Check test_extended_dsp/ and test_extended_nodsp/");
}
