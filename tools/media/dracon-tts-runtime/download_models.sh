#!/bin/bash
# Download TTS models for dracon-tts-runtime
# Run from crate root: ./download_models.sh [output_dir]

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTPUT_DIR="${1:-$SCRIPT_DIR/assets}"

mkdir -p "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/kokoro/voices"

echo "=== Downloading TTS Models ==="
echo "Output: $OUTPUT_DIR"

# Kitten TTS
echo ""
echo "[1/3] Downloading Kitten TTS (nano v0.8)..."
curl -L -o "$OUTPUT_DIR/kitten_tts_nano_v0_8.onnx" \
    "https://huggingface.co/rhasspy/kitten-tts/resolve/main/kitten_tts_nano_v0_8.onnx"

echo "[1/3] Downloading Kitten voices..."
curl -L -o "$OUTPUT_DIR/voices_v0_8.npz" \
    "https://huggingface.co/rhasspy/kitten-tts/resolve/main/voices_v0_8.npz"

# Kokoro TTS
echo ""
echo "[2/3] Downloading Kokoro TTS v1.0..."
curl -L -o "$OUTPUT_DIR/kokoro-v1.0.onnx" \
    "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/kokoro-v1.0.onnx"

echo "[2/3] Downloading Kokoro voices (key voices)..."
for voice in af_bella af_heart bm_lewis bm_daniel; do
    echo "  - $voice"
    curl -L -o "$OUTPUT_DIR/kokoro/voices/${voice}.bin" \
        "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/${voice}.bin"
done

# Optional: Download all 55 Kokoro voices
echo ""
echo "[3/3] Downloading all Kokoro voices (55 total)..."
for voice in af af_alloy af_aoede af_bella af_heart af_jessica af_kore af_nicole af_nova af_river af_sarah af_sky \
             am_adam am_echo am_eric am_fenrir am_liam am_michael am_onyx am_puck am_santa \
             bf_alice bf_emma bf_isabella bf_lily \
             bm_daniel bm_fable bm_george bm_lewis; do
    if [ ! -f "$OUTPUT_DIR/kokoro/voices/${voice}.bin" ]; then
        curl -L -o "$OUTPUT_DIR/kokoro/voices/${voice}.bin" \
            "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/${voice}.bin"
    fi
done

echo ""
echo "=== Done ==="
echo "Models downloaded to: $OUTPUT_DIR"
echo ""
echo "Sizes:"
du -sh "$OUTPUT_DIR"/*
