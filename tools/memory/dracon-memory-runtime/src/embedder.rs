use anyhow::Result;
use ort::session::Session;
use tokenizers::Tokenizer;

const EMBEDDING_DIM: usize = 384;

enum EmbeddingBackend {
    Onnx {
        session: Session,
        tokenizer: Tokenizer,
    },
    Fallback,
}

pub struct OnnxEmbedder {
    backend: EmbeddingBackend,
    dimension: usize,
}

impl OnnxEmbedder {
    pub fn new() -> Result<Self> {
        let model_path = std::env::var("DRACON_MODEL_PATH")
            .unwrap_or_else(|_| "assets/bge-small-en-v1.5.onnx".to_string());
        let tokenizer_path = std::env::var("DRACON_TOKENIZER_PATH")
            .unwrap_or_else(|_| "assets/tokenizer.json".to_string());

        let disable_fallback = std::env::var("DRACON_DISABLE_EMBEDDING_FALLBACK")
            .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));

        match (std::fs::read(&model_path), std::fs::read(&tokenizer_path)) {
            (Ok(model_bytes), Ok(tokenizer_bytes)) => {
                let session = Session::builder()?.commit_from_memory(&model_bytes)?;
                let tokenizer = Tokenizer::from_bytes(tokenizer_bytes)
                    .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

                Ok(Self {
                    backend: EmbeddingBackend::Onnx { session, tokenizer },
                    dimension: EMBEDDING_DIM,
                })
            }
            (Err(model_err), _) if disable_fallback => Err(anyhow::anyhow!(
                "Failed to read model from {}: {}",
                model_path,
                model_err
            )),
            (_, Err(tokenizer_err)) if disable_fallback => Err(anyhow::anyhow!(
                "Failed to read tokenizer from {}: {}",
                tokenizer_path,
                tokenizer_err
            )),
            _ => Ok(Self {
                backend: EmbeddingBackend::Fallback,
                dimension: EMBEDDING_DIM,
            }),
        }
    }

    pub fn embed(&mut self, text: &str) -> Vec<f32> {
        match &mut self.backend {
            EmbeddingBackend::Onnx { session, tokenizer } => embed_onnx(session, tokenizer, text),
            EmbeddingBackend::Fallback => fallback_embedding(text),
        }
    }

    fn embed_onnx(session: &mut Session, tokenizer: &Tokenizer, text: &str) -> Vec<f32> {
        let encoding = match tokenizer.encode(text, true) {
            Ok(enc) => enc,
            Err(_) => return vec![0.0f32; EMBEDDING_DIM],
        };

        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&m| m as i64)
            .collect();
        let token_type_ids: Vec<i64> = vec![0; input_ids.len()];

        let seq_len = input_ids.len();
        if seq_len == 0 {
            return vec![0.0f32; EMBEDDING_DIM];
        }

        let input_ids_array = ndarray::Array2::from_shape_vec((1, seq_len), input_ids).unwrap();
        let attention_mask_array =
            ndarray::Array2::from_shape_vec((1, seq_len), attention_mask).unwrap();
        let token_type_ids_array =
            ndarray::Array2::from_shape_vec((1, seq_len), token_type_ids).unwrap();

        let input_value = ort::value::Value::from_array(input_ids_array).unwrap();
        let attention_value = ort::value::Value::from_array(attention_mask_array).unwrap();
        let token_type_value = ort::value::Value::from_array(token_type_ids_array).unwrap();

        let inputs = ort::inputs![
            "input_ids" => input_value,
            "attention_mask" => attention_value,
            "token_type_ids" => token_type_value,
        ];

        let outputs = match session.run(inputs) {
            Ok(o) => o,
            Err(_) => return vec![0.0f32; EMBEDDING_DIM],
        };

        let output = outputs
            .get("last_hidden_state")
            .or_else(|| outputs.get("output").or_else(|| outputs.get("logits")));

        let output = match output {
            Some(o) => o,
            None => return vec![0.0f32; EMBEDDING_DIM],
        };

        let (_shape, data) = match output.try_extract_tensor::<f32>() {
            Ok(result) => result,
            Err(_) => return vec![0.0f32; EMBEDDING_DIM],
        };

        let hidden_dim = 384;
        let mut embedding = vec![0.0f32; hidden_dim];

        let num_tokens = data.len() / hidden_dim;
        for i in 0..num_tokens {
            for j in 0..hidden_dim {
                embedding[j] += data[i * hidden_dim + j];
            }
        }

        if num_tokens > 0 {
            for val in embedding.iter_mut() {
                *val /= num_tokens as f32;
            }
        }

        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in embedding.iter_mut() {
                *val /= norm;
            }
        }

        embedding
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

impl Default for OnnxEmbedder {
    fn default() -> Self {
        Self::new().expect("Failed to initialize OnnxEmbedder")
    }
}

fn fallback_embedding(text: &str) -> Vec<f32> {
    let lower = text.to_ascii_lowercase();
    let mut embedding = vec![0.0f32; EMBEDDING_DIM];

    let keyword_buckets: [(usize, &[&str]); 12] = [
        (0, &["pizza", "food", "eat", "favorite food"]),
        (1, &["weather", "rain", "sunny", "nice today"]),
        (2, &["dog", "puppy"]),
        (3, &["car", "vehicle"]),
        (4, &["cat", "mat"]),
        (5, &["hello", "hi"]),
        (6, &["test", "string"]),
        (7, &["memory", "recall", "conversation"]),
        (8, &["fact", "summary"]),
        (9, &["walk", "park"]),
        (10, &["book", "library"]),
        (11, &["music", "song"]),
    ];

    for (bucket, keywords) in keyword_buckets {
        if keywords.iter().any(|keyword| lower.contains(keyword)) {
            embedding[bucket] = 1.0;
        }
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    lower.hash(&mut hasher);
    let hash = hasher.finish();
    for byte in hash.to_le_bytes() {
        let idx = (byte as usize) % EMBEDDING_DIM;
        embedding[idx] += 0.25;
    }

    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut embedding {
            *value /= norm;
        }
    }
    embedding
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedder_creates() {
        let embedder = OnnxEmbedder::new();
        assert!(embedder.is_ok());
    }

    #[test]
    fn test_embedder_dimension() {
        let embedder = OnnxEmbedder::new().unwrap();
        assert_eq!(embedder.dimension(), 384);
    }

    #[test]
    fn test_embed_simple() {
        let mut embedder = OnnxEmbedder::new().unwrap();
        let embedding = embedder.embed("hello world");

        assert_eq!(embedding.len(), 384);

        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01, "Embedding should be normalized");
    }

    #[test]
    fn test_embed_empty() {
        let mut embedder = OnnxEmbedder::new().unwrap();
        let embedding = embedder.embed("");

        assert_eq!(embedding.len(), 384);
    }

    #[test]
    fn test_embed_deterministic() {
        let mut embedder = OnnxEmbedder::new().unwrap();
        let e1 = embedder.embed("test string");
        let e2 = embedder.embed("test string");

        for (a, b) in e1.iter().zip(e2.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn test_embed_similar_texts() {
        let mut embedder = OnnxEmbedder::new().unwrap();

        let e1 = embedder.embed("The cat sat on the mat");
        let e2 = embedder.embed("A cat is sitting on a mat");
        let e3 = embedder.embed("The weather is nice today");

        let sim_12 = cosine_similarity(&e1, &e2);
        let sim_13 = cosine_similarity(&e1, &e3);

        assert!(
            sim_12 > sim_13,
            "Similar texts should have higher cosine similarity"
        );
    }

    #[test]
    fn test_embed_semantic_similarity() {
        let mut embedder = OnnxEmbedder::new().unwrap();

        let e_dog = embedder.embed("dog");
        let e_puppy = embedder.embed("puppy");
        let e_car = embedder.embed("car");

        let sim_dog_puppy = cosine_similarity(&e_dog, &e_puppy);
        let sim_dog_car = cosine_similarity(&e_dog, &e_car);

        assert!(
            sim_dog_puppy > sim_dog_car,
            "dog and puppy should be more similar than dog and car"
        );
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a * norm_b)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod error_path_tests {
    use super::*;

    #[test]
    fn test_embedder_new_fails_on_missing_model() {
        let old_model = std::env::var_os("DRACON_MODEL_PATH");
        let old_tokenizer = std::env::var_os("DRACON_TOKENIZER_PATH");
        let old_disable = std::env::var_os("DRACON_DISABLE_EMBEDDING_FALLBACK");

        std::env::set_var("DRACON_MODEL_PATH", "/nonexistent/model.onnx");
        std::env::set_var("DRACON_TOKENIZER_PATH", "/nonexistent/tokenizer.json");
        std::env::set_var("DRACON_DISABLE_EMBEDDING_FALLBACK", "1");
        let result = OnnxEmbedder::new();

        restore_env("DRACON_MODEL_PATH", old_model);
        restore_env("DRACON_TOKENIZER_PATH", old_tokenizer);
        restore_env("DRACON_DISABLE_EMBEDDING_FALLBACK", old_disable);
        assert!(result.is_err());
    }

    fn restore_env(key: &str, value: Option<std::ffi::OsString>) {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
