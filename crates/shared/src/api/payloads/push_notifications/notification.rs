use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub body: Option<String>,
    pub icon: Option<String>,
    pub sent: DateTime<Utc>,
}
