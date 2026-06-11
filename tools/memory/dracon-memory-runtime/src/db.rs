use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;
use sqlite_vec::sqlite3_vec_init;

use crate::memory_contracts::{Conversation, Role, UserFact};

const EMBEDDING_DIM: usize = 384;

/// SQLite-backed persistence layer for memory conversations and facts.
pub struct MemoryDb {
    conn: Connection,
}

impl MemoryDb {
    /// Open or create a SQLite database at `path`.
    pub fn new(path: &str) -> Result<Self> {
        let init_fn: unsafe extern "C" fn() = sqlite3_vec_init;
        // SAFETY: sqlite3_vec_init is a C function pointer with the correct signature
        // for sqlite3_auto_extension. It registers a valid SQLite extension and does not
        // create aliased mutable state or perform operations requiring a specific runtime.
        unsafe {
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute::<
                unsafe extern "C" fn(),
                unsafe extern "C" fn(
                    *mut rusqlite::ffi::sqlite3,
                    *mut *mut i8,
                    *const rusqlite::ffi::sqlite3_api_routines,
                ) -> i32,
            >(init_fn)));
        }

        let conn = if path == ":memory:" {
            Connection::open_in_memory()?
        } else {
            Connection::open(path)?
        };

        let mut db = Self { conn };
        db.initialize()?;

        Ok(db)
    }

    fn initialize(&mut self) -> Result<()> {
        self.conn.execute_batch(&format!(
            r#"
                CREATE TABLE IF NOT EXISTS conversations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                    role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
                    content TEXT NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_conversations_time 
                ON conversations(timestamp DESC);

                CREATE VIRTUAL TABLE IF NOT EXISTS vec_conversations 
                USING vec0(
                    embedding float[{EMBEDDING_DIM}]
                );

                CREATE TABLE IF NOT EXISTS user_facts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    category TEXT NOT NULL,
                    key TEXT NOT NULL,
                    value TEXT NOT NULL,
                    confidence REAL DEFAULT 1.0,
                    source TEXT,
                    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                    UNIQUE(category, key)
                );

                CREATE INDEX IF NOT EXISTS idx_facts_category ON user_facts(category);
            "#
        ))?;

        Ok(())
    }

    fn embedding_to_bytes(embedding: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(embedding.len() * 4);
        for &val in embedding {
            bytes.extend_from_slice(&val.to_le_bytes());
        }
        bytes
    }

    /// Store a conversation and its embedding, returning the database row id.
    pub fn store_conversation(&self, role: Role, content: &str, embedding: &[f32]) -> Result<i64> {
        let tx = self.conn.unchecked_transaction()?;

        let conversation_id: i64 = tx.query_row(
            "INSERT INTO conversations (role, content) VALUES (?1, ?2) RETURNING id",
            [role.as_str(), content],
            |row| row.get(0),
        )?;

        let embedding_bytes = Self::embedding_to_bytes(embedding);
        tx.execute(
            "INSERT INTO vec_conversations (rowid, embedding) VALUES (?1, ?2)",
            rusqlite::params![conversation_id, embedding_bytes],
        )?;

        tx.commit()?;

        Ok(conversation_id)
    }

    /// Search for conversations similar to `query_embedding`.
    pub fn search_similar(&self, query_embedding: &[f32], k: usize) -> Result<Vec<Conversation>> {
        let embedding_bytes = Self::embedding_to_bytes(query_embedding);

        let mut stmt = self.conn.prepare(
            r#"
                SELECT 
                    c.id, c.timestamp, c.role, c.content
                FROM vec_conversations v
                JOIN conversations c ON v.rowid = c.id
                WHERE v.embedding MATCH ?1 AND k = ?2
                ORDER BY distance
            "#,
        )?;

        let conversations = stmt
            .query_map(rusqlite::params![embedding_bytes, k as i32], |row| {
                Ok(Conversation {
                    id: row.get(0)?,
                    timestamp: row
                        .get::<_, String>(1)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                    role: Role::parse(&row.get::<_, String>(2)?).unwrap_or(Role::User),
                    content: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(conversations)
    }

    /// Return the most recent conversations.
    pub fn get_recent(&self, limit: usize) -> Result<Vec<Conversation>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, role, content FROM conversations 
             ORDER BY timestamp DESC LIMIT ?1",
        )?;

        let conversations = stmt
            .query_map([limit as i32], |row| {
                Ok(Conversation {
                    id: row.get(0)?,
                    timestamp: row
                        .get::<_, String>(1)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                    role: Role::parse(&row.get::<_, String>(2)?).unwrap_or(Role::User),
                    content: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(conversations)
    }

    /// Delete all stored conversations and vector rows.
    pub fn clear(&self) -> Result<()> {
        self.conn.execute("DELETE FROM conversations", [])?;
        self.conn.execute("DELETE FROM vec_conversations", [])?;
        Ok(())
    }

    /// Store or update a user fact.
    pub fn store_fact(
        &self,
        category: &str,
        key: &str,
        value: &str,
        source: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO user_facts (category, key, value, source, updated_at)
            VALUES (?1, ?2, ?3, ?4, datetime('now'))
            ON CONFLICT(category, key) DO UPDATE SET 
                value = excluded.value,
                source = excluded.source,
                updated_at = datetime('now')
            "#,
            rusqlite::params![category, key, value, source],
        )?;
        Ok(())
    }

    /// Return one user fact by category and key.
    pub fn get_fact(&self, category: &str, key: &str) -> Result<Option<UserFact>> {
        let mut stmt = self.conn.prepare(
            "SELECT category, key, value, confidence, source 
             FROM user_facts WHERE category = ?1 AND key = ?2",
        )?;

        let result = stmt.query_row(rusqlite::params![category, key], |row| {
            Ok(UserFact::new(
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            )
            .with_confidence(row.get::<_, f32>(3)?)
            .with_source(row.get::<_, Option<String>>(4)?.unwrap_or_default()))
        });

        match result {
            Ok(fact) => Ok(Some(fact)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Return all user facts in a category.
    pub fn get_facts_by_category(&self, category: &str) -> Result<Vec<UserFact>> {
        let mut stmt = self.conn.prepare(
            "SELECT category, key, value, confidence, source 
             FROM user_facts WHERE category = ?1 ORDER BY updated_at DESC",
        )?;

        let facts = stmt
            .query_map([category], |row| {
                Ok(UserFact::new(
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                )
                .with_confidence(row.get::<_, f32>(3)?)
                .with_source(row.get::<_, Option<String>>(4)?.unwrap_or_default()))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(facts)
    }

    /// Return all stored user facts.
    pub fn get_all_facts(&self) -> Result<Vec<UserFact>> {
        let mut stmt = self.conn.prepare(
            "SELECT category, key, value, confidence, source 
             FROM user_facts ORDER BY updated_at DESC",
        )?;

        let facts = stmt
            .query_map([], |row| {
                Ok(UserFact::new(
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                )
                .with_confidence(row.get::<_, f32>(3)?)
                .with_source(row.get::<_, Option<String>>(4)?.unwrap_or_default()))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(facts)
    }
}

impl MemoryDb {
    /// Return a text summary of all stored user facts.
    pub fn get_all_facts_summary(&self) -> Result<String> {
        let facts = self.get_all_facts()?;
        if facts.is_empty() {
            return Ok("No known user information.".to_string());
        }

        let mut summary = String::from("Known user information:\n");
        let mut categories: std::collections::HashMap<String, Vec<(String, String)>> =
            std::collections::HashMap::new();

        for fact in facts {
            categories
                .entry(fact.category.clone())
                .or_default()
                .push((fact.key, fact.value));
        }

        for (category, entries) in categories {
            summary.push_str(&format!("\n{}:\n", category));
            for (key, value) in entries {
                summary.push_str(&format!("  - {}: {}\n", key, value));
            }
        }

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_embedding(values: &[f32]) -> Vec<f32> {
        let mut emb = vec![0.0f32; EMBEDDING_DIM];
        for (i, &v) in values.iter().enumerate() {
            if i < EMBEDDING_DIM {
                emb[i] = v;
            }
        }
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in emb.iter_mut() {
                *val /= norm;
            }
        }
        emb
    }

    #[test]
    fn test_memory_db_new_in_memory() {
        let db = MemoryDb::new(":memory:");
        assert!(db.is_ok());
    }

    #[test]
    fn test_memory_db_store_conversation() {
        let db = MemoryDb::new(":memory:").unwrap();
        let embedding = create_test_embedding(&[0.5, 0.5, 0.5]);

        let id = db.store_conversation(Role::User, "Hello world", &embedding);
        assert!(id.is_ok());
        assert!(id.unwrap() > 0);
    }

    #[test]
    fn test_memory_db_get_recent() {
        let db = MemoryDb::new(":memory:").unwrap();
        let embedding = create_test_embedding(&[0.5]);

        db.store_conversation(Role::User, "Message 1", &embedding)
            .unwrap();
        db.store_conversation(Role::User, "Message 2", &embedding)
            .unwrap();

        let recent = db.get_recent(10).unwrap();
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_memory_db_clear() {
        let db = MemoryDb::new(":memory:").unwrap();
        let embedding = create_test_embedding(&[0.5]);

        db.store_conversation(Role::User, "Test", &embedding)
            .unwrap();
        assert_eq!(db.get_recent(10).unwrap().len(), 1);

        db.clear().unwrap();
        assert_eq!(db.get_recent(10).unwrap().len(), 0);
    }

    #[test]
    fn test_memory_db_store_fact() {
        let db = MemoryDb::new(":memory:").unwrap();

        let result = db.store_fact("personal", "name", "Alice", None);
        assert!(result.is_ok());

        let fact = db.get_fact("personal", "name").unwrap();
        assert!(fact.is_some());
        assert_eq!(fact.unwrap().value, "Alice");
    }

    #[test]
    fn test_memory_db_get_all_facts() {
        let db = MemoryDb::new(":memory:").unwrap();

        db.store_fact("personal", "name", "Alice", None).unwrap();
        db.store_fact("preferences", "color", "blue", None).unwrap();

        let facts = db.get_all_facts().unwrap();
        assert_eq!(facts.len(), 2);
    }

    #[test]
    fn test_memory_db_get_facts_by_category() {
        let db = MemoryDb::new(":memory:").unwrap();

        db.store_fact("personal", "name", "Alice", None).unwrap();
        db.store_fact("personal", "city", "Boston", None).unwrap();
        db.store_fact("preferences", "color", "blue", None).unwrap();

        let personal_facts = db.get_facts_by_category("personal").unwrap();
        assert_eq!(personal_facts.len(), 2);
    }

    #[test]
    fn test_memory_db_fact_upsert() {
        let db = MemoryDb::new(":memory:").unwrap();

        db.store_fact("personal", "name", "Alice", None).unwrap();
        db.store_fact("personal", "name", "Bob", None).unwrap();

        let facts = db.get_all_facts().unwrap();
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].value, "Bob");
    }

    #[test]
    fn test_memory_db_get_all_facts_summary() {
        let db = MemoryDb::new(":memory:").unwrap();

        db.store_fact("personal", "name", "Alice", None).unwrap();
        db.store_fact("preferences", "color", "blue", None).unwrap();

        let summary = db.get_all_facts_summary().unwrap();
        assert!(summary.contains("Alice"));
        assert!(summary.contains("blue"));
    }

    #[test]
    fn test_memory_db_search_similar() {
        let db = MemoryDb::new(":memory:").unwrap();

        let e1 = create_test_embedding(&[1.0, 0.0, 0.0]);
        let e2 = create_test_embedding(&[0.0, 1.0, 0.0]);
        let e3 = create_test_embedding(&[0.0, 0.0, 1.0]);

        db.store_conversation(Role::User, "About cats", &e1)
            .unwrap();
        db.store_conversation(Role::User, "About dogs", &e2)
            .unwrap();
        db.store_conversation(Role::User, "About birds", &e3)
            .unwrap();

        let query = create_test_embedding(&[1.0, 0.0, 0.0]);
        let results = db.search_similar(&query, 2).unwrap();

        assert!(results.len() <= 2);
    }
}
