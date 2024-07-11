use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sdp(String);

impl From<String> for Sdp {
    fn from(value: String) -> Self {
        Sdp(value)
    }
}

impl From<Sdp> for String {
    fn from(value: Sdp) -> Self {
        value.0
    }
}

impl Deref for Sdp {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Signal {
    Offer(Sdp),
    Answer(Sdp),
    IceCandidate(),
}
