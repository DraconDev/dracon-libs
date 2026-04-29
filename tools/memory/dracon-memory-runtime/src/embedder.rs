use anyhow::Result;
use ort::session::Session;
use tokenizers::Tokenizer;

const EMBEDDING_DIM: usize = 384;

pub struct OnnxEmbedder {
    session: Session,
    tokenizer: Tokenizer,
    dimension: usize,
}

impl OnnxEmbedder {
    pub fn new() -> Result<Self> {
        let model_path = std::env::var("DRACON_MODEL_PATH")
            .unwrap_or_else(|_| "assets/bge-small-en-v1.5.onnx".to_string());
        let tokenizer_path = std::env::var("DRACON_TOKENIZER_PATH")
            .unwrap_or_else(|_| "assets/tokenizer.json".to_string());

        let model_bytes = std::fs::read(&model_path)
            .map_err(|e| anyhow::anyhow!("Failed to read model from {}: {}", model_path, e))?;
        let tokenizer_bytes = std::fs::read(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to read tokenizer from {}: {}", tokenizer_path, e))?;

        let session = Session::builder()?.commit_from_memory(model_bytes)?;

        let tokenizer = Tokenizer::from_bytes(tokenizer_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        Ok(Self {
            session,
            tokenizer,
            dimension: EMBEDDING_DIM,
        })
    }

    pub fn embed(&mut self, text: &str) -> Vec<f32> {
        let encoding = match self.tokenizer.encode(text, true) {
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

        let outputs = match self.session.run(inputs) {
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
