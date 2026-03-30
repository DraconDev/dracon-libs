#!/bin/bash
# Download STT models for dracon-stt-runtime
# Run from crate root: ./download_models.sh [output_dir]

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTPUT_DIR="${1:-$SCRIPT_DIR/assets}"

mkdir -p "$OUTPUT_DIR"

echo "=== Downloading STT Models ==="
echo "Output: $OUTPUT_DIR"

# Parakeet CTC
echo ""
echo "[1/2] Downloading Parakeet CTC 0.6B (int8)..."
mkdir -p "$OUTPUT_DIR/parakeet-ctc/onnx"
curl -L -o "$OUTPUT_DIR/parakeet-ctc/onnx/model_int8.onnx" \
    "https://huggingface.co/nvidia/parakeet-ctc-0.6b/resolve/main/onnx/model_int8.onnx"

# VAD (optional, useful for voice activity detection)
echo ""
echo "[2/2] Downloading Silero VAD..."
curl -L -o "$OUTPUT_DIR/silero_vad.onnx" \
    "https://github.com/snakers4/silero-vad/raw/master/files/silero_vad.onnx"

echo ""
echo "=== Done ==="
echo "Models downloaded to: $OUTPUT_DIR"
echo ""
echo "Sizes:"
du -sh "$OUTPUT_DIR"/*
