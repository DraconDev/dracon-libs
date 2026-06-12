use serde::{Deserialize, Serialize};

/// Configuration for desktop notification behavior per event type.
#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationConfig {
    /// Whether notifications are enabled at all.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Notify on task completion events.
    #[serde(default = "default_true")]
    pub on_task_complete: bool,
    /// Notify on manifestation (workspace change) events.
    #[serde(default = "default_true")]
    pub on_manifestation: bool,
    /// Notify on security alert events.
    #[serde(default = "default_true")]
    pub on_security_alert: bool,
    /// Notify on error events.
    #[serde(default = "default_true")]
    pub on_error: bool,
    /// Notify on sync events (default: off).
    #[serde(default)]
    pub on_sync: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            on_task_complete: true,
            on_manifestation: true,
            on_security_alert: true,
            on_error: true,
            on_sync: false,
        }
    }
}

fn default_true() -> bool {
    true
}
