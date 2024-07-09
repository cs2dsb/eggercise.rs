use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rtc;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash)]
pub struct RoomId(Uuid);

impl RoomId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl rtc::RoomId for RoomId {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash)]
pub struct PeerId(Uuid);

impl PeerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl rtc::PeerId for PeerId {}
