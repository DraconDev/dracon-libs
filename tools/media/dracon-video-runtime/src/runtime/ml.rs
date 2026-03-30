//! ML-based video processing (face detection, person segmentation)

use anyhow::Result;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tract_onnx::prelude::*;

pub use crate::protocol::video::FaceRegion;

/// Frame extraction utilities
pub struct FrameExtractor;

impl FrameExtractor {
    pub fn extract_frames(
        video_path: &Path,
        output_dir: &Path,
        interval_fps: f32,
    ) -> Result<Vec<std::path::PathBuf>> {
        std::fs::create_dir_all(output_dir)?;

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                video_path.to_str().unwrap_or(""),
                "-vf",
                &format!("fps={}", interval_fps),
                "-y",
                &format!("{}/frame_%04d.png", output_dir.to_str().unwrap_or("")),
            ])
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to extract frames from video");
        }

        let mut frames = vec![];
        for entry in std::fs::read_dir(output_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "png").unwrap_or(false) {
                frames.push(path);
            }
        }

        frames.sort();
        Ok(frames)
    }

    pub fn get_video_dimensions(video_path: &Path) -> Result<(u32, u32)> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=width,height",
                "-of",
                "csv=p=0",
                video_path.to_str().unwrap_or(""),
            ])
            .output()?;

        let dims = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = dims.trim().split(',').collect();

        if parts.len() == 2 {
            let width: u32 = parts[0].parse()?;
            let height: u32 = parts[1].parse()?;
            Ok((width, height))
        } else {
            anyhow::bail!("Failed to parse video dimensions")
        }
    }

    pub fn get_video_duration(video_path: &Path) -> Result<f32> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                video_path.to_str().unwrap_or(""),
            ])
            .output()?;

        let duration = String::from_utf8_lossy(&output.stdout);
        duration
            .trim()
            .parse::<f32>()
            .map_err(|e| anyhow::anyhow!("Failed to parse duration: {}", e))
    }
}

const FACE_MODEL_ID: &str = "onnx-models/ultra-light-face-detector";
const FACE_MODEL_FILE: &str = "version-RFB-320.onnx";

#[allow(dead_code)]
const SEGMENT_MODEL_ID: &str = "dhkim2810/MODNet";
#[allow(dead_code)]
const SEGMENT_MODEL_FILE: &str = "modnet.onnx";

type OnnxModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

pub struct FaceDetector {
    model: Arc<OnnxModel>,
}

impl FaceDetector {
    pub fn load() -> Result<Self> {
        let model_path = Self::get_model_path()?;

        if !model_path.exists() {
            Self::download_model(&model_path)?;
        }

        let model = tract_onnx::onnx()
            .model_for_path(&model_path)?
            .into_optimized()?
            .into_runnable()?;

        Ok(Self {
            model: Arc::new(model),
        })
    }

    fn get_model_path() -> Result<std::path::PathBuf> {
        let cache_dir = directories::ProjectDirs::from("com", "dracon", "dracon-video-runtime")
            .map(|dirs| dirs.cache_dir().to_path_buf())
            .unwrap_or_else(std::env::temp_dir);

        Ok(cache_dir.join("face_detection.onnx"))
    }

    fn download_model(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        tracing::info!("Downloading face detection model from HuggingFace...");

        let api = hf_hub::api::sync::Api::new()?;
        let repo = api.model(FACE_MODEL_ID.to_string());
        let downloaded = repo.get(FACE_MODEL_FILE)?;

        std::fs::copy(&downloaded, path)?;

        tracing::info!(path = ?path, "Model downloaded");
        Ok(())
    }

    pub fn detect(&self, frame: &image::DynamicImage) -> Result<Vec<FaceBox>> {
        let input = Self::preprocess(frame)?;
        let result = self.model.run(tvec!(input.into()))?;
        Self::parse_output(&result)
    }

    fn preprocess(image: &image::DynamicImage) -> Result<Tensor> {
        let resized = image.resize_exact(320, 320, image::imageops::FilterType::Triangle);
        let rgb = resized.to_rgb8();
        let data: Vec<f32> = rgb
            .pixels()
            .flat_map(|p| p.0.iter().map(|&v| v as f32 / 255.0))
            .collect();

        let tensor = Tensor::from_shape(&[1, 3, 320, 320], &data)?;
        Ok(tensor)
    }

    fn parse_output(output: &[TValue]) -> Result<Vec<FaceBox>> {
        if output.len() < 2 {
            return Ok(vec![]);
        }

        let scores = output[0].to_array_view::<f32>()?;
        let boxes = output[1].to_array_view::<f32>()?;

        let confidence_threshold = 0.5;
        let mut faces = Vec::new();

        let score_dims = scores.shape();
        let num_faces = if score_dims.len() == 2 || score_dims.len() == 3 {
            score_dims[1]
        } else {
            return Ok(vec![]);
        };

        let box_dims = boxes.shape();
        let boxes_are_flat = box_dims.len() == 2 && box_dims[1] == num_faces * 4;

        for i in 0..num_faces {
            let score = scores[i];
            if score < confidence_threshold {
                continue;
            }

            let (x1, y1, x2, y2) = if boxes_are_flat || box_dims.len() == 3 {
                (
                    boxes[i * 4],
                    boxes[i * 4 + 1],
                    boxes[i * 4 + 2],
                    boxes[i * 4 + 3],
                )
            } else {
                continue;
            };

            let x = x1.clamp(0.0, 1.0);
            let y = y1.clamp(0.0, 1.0);
            let width = (x2 - x1).clamp(0.0, 1.0 - x);
            let height = (y2 - y1).clamp(0.0, 1.0 - y);

            faces.push(FaceBox {
                x,
                y,
                width,
                height,
                confidence: score,
            });
        }

        faces.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(faces)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FaceBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
}

impl FaceBox {
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

pub struct CropRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl CropRegion {
    pub fn center_crop_9_16() -> Self {
        let crop_width = 9.0 / 16.0;
        Self {
            x: (1.0 - crop_width) / 2.0,
            y: 0.0,
            width: crop_width,
            height: 1.0,
        }
    }

    pub fn from_face(face: &FaceBox, video_aspect: f32) -> Self {
        let target_aspect = 9.0 / 16.0;
        let crop_width = target_aspect / video_aspect;
        let face_center_x = face.x + face.width / 2.0;
        let mut crop_x = face_center_x - crop_width / 2.0;
        crop_x = crop_x.max(0.0).min(1.0 - crop_width);

        Self {
            x: crop_x,
            y: 0.0,
            width: crop_width,
            height: 1.0,
        }
    }
}

pub struct AutoReframeProcessor {
    detector: FaceDetector,
}

impl AutoReframeProcessor {
    pub fn new() -> Result<Self> {
        let detector = FaceDetector::load()?;
        Ok(Self { detector })
    }

    pub fn analyze_video(
        &self,
        video_path: &Path,
        sample_fps: f32,
    ) -> Result<Vec<(f32, CropRegion)>> {
        let temp_dir = std::env::temp_dir().join("dracon-frames");
        let frames = FrameExtractor::extract_frames(video_path, &temp_dir, sample_fps)?;

        let (video_width, video_height) = FrameExtractor::get_video_dimensions(video_path)?;
        let video_aspect = video_width as f32 / video_height as f32;

        let mut crop_regions = Vec::new();

        for (i, frame_path) in frames.iter().enumerate() {
            let timestamp = (i as f32) / sample_fps;
            let frame = image::open(frame_path)?;
            let faces = self.detector.detect(&frame)?;

            let crop = if let Some(main_face) = faces.first() {
                CropRegion::from_face(main_face, video_aspect)
            } else {
                CropRegion::center_crop_9_16()
            };

            crop_regions.push((timestamp, crop));
        }

        for frame in &frames {
            let _ = std::fs::remove_file(frame);
        }

        Ok(crop_regions)
    }

    pub fn generate_crop_filter(
        &self,
        crop_regions: &[(f32, CropRegion)],
        video_width: u32,
        video_height: u32,
    ) -> String {
        if crop_regions.is_empty() {
            return "crop=ih*9/16:ih,scale=1080:1920".to_string();
        }

        let region = &crop_regions[0].1;
        let crop_w = (region.width * video_width as f32) as u32;
        let crop_h = video_height;
        let crop_x = (region.x * video_width as f32) as u32;
        let crop_y = 0u32;

        format!(
            "crop={}:{}:{}:{},scale=1080:1920",
            crop_w, crop_h, crop_x, crop_y
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_box_center() {
        let face = FaceBox {
            x: 0.1,
            y: 0.2,
            width: 0.3,
            height: 0.4,
            confidence: 0.9,
        };

        let (cx, cy) = face.center();
        assert!((cx - 0.25).abs() < 0.001);
        assert!((cy - 0.4).abs() < 0.001);
    }
}
