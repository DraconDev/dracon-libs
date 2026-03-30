use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub on_task_complete: bool,
    #[serde(default = "default_true")]
    pub on_manifestation: bool,
    #[serde(default = "default_true")]
    pub on_security_alert: bool,
    #[serde(default = "default_true")]
    pub on_error: bool,
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
