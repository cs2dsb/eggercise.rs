use serde::{Deserialize, Serialize};

use super::SdpType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    pub type_: SdpType,
    pub sdp: String,
}
