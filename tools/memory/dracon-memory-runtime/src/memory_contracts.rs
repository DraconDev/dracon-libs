use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }

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

#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub role: Role,
    pub content: String,
}

impl Conversation {
    pub fn new(id: i64, role: Role, content: String) -> Self {
        Self {
            id,
            timestamp: Utc::now(),
            role,
            content,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserFact {
    pub category: String,
    pub key: String,
    pub value: String,
    pub confidence: f32,
    pub source: Option<String>,
}

impl UserFact {
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

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

pub trait TextEmbedder: Send + Sync {
    fn embed(&self, text: &str) -> Vec<f32>;
    fn dimension(&self) -> usize;
}

pub trait MemoryStore: Send + Sync {
    fn store_conversation(
        &self,
        role: Role,
        content: &str,
        embedding: &[f32],
    ) -> anyhow::Result<i64>;
    fn search_similar(
        &self,
        query_embedding: &[f32],
        k: usize,
    ) -> anyhow::Result<Vec<Conversation>>;
    fn get_recent(&self, limit: usize) -> anyhow::Result<Vec<Conversation>>;
    fn clear(&self) -> anyhow::Result<()>;

    fn store_fact(
        &self,
        category: &str,
        key: &str,
        value: &str,
        source: Option<&str>,
    ) -> anyhow::Result<()>;
    fn get_fact(&self, category: &str, key: &str) -> anyhow::Result<Option<UserFact>>;
    fn get_facts_by_category(&self, category: &str) -> anyhow::Result<Vec<UserFact>>;
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
