use chrono::{DateTime, Utc};
use std::fmt;

/// Runtime-side conversation role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// User-authored message.
    User,
    /// Assistant-authored message.
    Assistant,
}

impl Role {
    /// Return the stable database string for this role.
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }

    /// Parse a role from its stable database string.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "user" => Some(Role::User),
            "assistant" => Some(Role::Assistant),
            _ => None,
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Stored conversation row returned by the runtime.
#[derive(Debug, Clone)]
pub struct Conversation {
    /// Database row id.
    pub id: i64,
    /// Timestamp assigned when the row is materialized.
    pub timestamp: DateTime<Utc>,
    /// Conversation participant role.
    pub role: Role,
    /// Message text.
    pub content: String,
}

impl Conversation {
    /// Create a new conversation with the current timestamp.
    pub fn new(id: i64, role: Role, content: String) -> Self {
        Self {
            id,
            timestamp: Utc::now(),
            role,
            content,
        }
    }
}

/// User fact stored outside the conversation stream.
#[derive(Debug, Clone)]
pub struct UserFact {
    /// Fact category.
    pub category: String,
    /// Fact key within its category.
    pub key: String,
    /// Fact value.
    pub value: String,
    /// Confidence score for the fact.
    pub confidence: f32,
    /// Optional source describing where the fact came from.
    pub source: Option<String>,
}

impl UserFact {
    /// Create a new fact with default confidence and no source.
    pub fn new(
        category: impl Into<String>,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            category: category.into(),
            key: key.into(),
            value: value.into(),
            confidence: 1.0,
            source: None,
        }
    }

    /// Set an explicit confidence score.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    /// Set the fact source.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

/// Synchronous text embedding contract implemented by runtime embedders.
pub trait TextEmbedder: Send + Sync {
    /// Embed `text` into a fixed-dimensional vector.
    fn embed(&self, text: &str) -> Vec<f32>;
    /// Return the embedding dimension.
    fn dimension(&self) -> usize;
}

/// Synchronous storage contract implemented by runtime databases.
pub trait MemoryStore: Send + Sync {
    /// Store one conversation with a precomputed embedding.
    fn store_conversation(
        &self,
        role: Role,
        content: &str,
        embedding: &[f32],
    ) -> anyhow::Result<i64>;
    /// Search for conversations similar to `query_embedding`.
    fn search_similar(
        &self,
        query_embedding: &[f32],
        k: usize,
    ) -> anyhow::Result<Vec<Conversation>>;
    /// Return the most recent conversation rows.
    fn get_recent(&self, limit: usize) -> anyhow::Result<Vec<Conversation>>;
    /// Delete all stored conversations and vector rows.
    fn clear(&self) -> anyhow::Result<()>;

    /// Store or update a user fact.
    fn store_fact(
        &self,
        category: &str,
        key: &str,
        value: &str,
        source: Option<&str>,
    ) -> anyhow::Result<()>;
    /// Return one user fact by category and key.
    fn get_fact(&self, category: &str, key: &str) -> anyhow::Result<Option<UserFact>>;
    /// Return all user facts in a category.
    fn get_facts_by_category(&self, category: &str) -> anyhow::Result<Vec<UserFact>>;
    /// Return all stored user facts.
    fn get_all_facts(&self) -> anyhow::Result<Vec<UserFact>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_as_str() {
        assert_eq!(Role::User.as_str(), "user");
        assert_eq!(Role::Assistant.as_str(), "assistant");
    }

    #[test]
    fn test_role_parse() {
        assert_eq!(Role::parse("user"), Some(Role::User));
        assert_eq!(Role::parse("assistant"), Some(Role::Assistant));
        assert_eq!(Role::parse("unknown"), None);
    }

    #[test]
    fn test_role_display() {
        assert_eq!(format!("{}", Role::User), "user");
        assert_eq!(format!("{}", Role::Assistant), "assistant");
    }

    #[test]
    fn test_role_equality() {
        assert_eq!(Role::User, Role::User);
        assert_ne!(Role::User, Role::Assistant);
    }

    #[test]
    fn test_conversation_new() {
        let conv = Conversation::new(1, Role::User, "Hello".to_string());
        assert_eq!(conv.id, 1);
        assert_eq!(conv.role, Role::User);
        assert_eq!(conv.content, "Hello");
    }

    #[test]
    fn test_user_fact_new() {
        let fact = UserFact::new("personal", "name", "Alice");
        assert_eq!(fact.category, "personal");
        assert_eq!(fact.key, "name");
        assert_eq!(fact.value, "Alice");
        assert_eq!(fact.confidence, 1.0);
        assert!(fact.source.is_none());
    }

    #[test]
    fn test_user_fact_builder() {
        let fact = UserFact::new("personal", "name", "Alice")
            .with_confidence(0.9)
            .with_source("user_message");

        assert_eq!(fact.confidence, 0.9);
        assert_eq!(fact.source, Some("user_message".to_string()));
    }
}
