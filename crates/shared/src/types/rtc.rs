use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rtc;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub struct RoomId(Uuid);

impl From<Uuid> for RoomId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl RoomId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl rtc::RoomId for RoomId {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub struct PeerId(Uuid);

impl Default for PeerId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.as_hyphenated())
    }
}

impl PeerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl rtc::PeerId for PeerId {}
