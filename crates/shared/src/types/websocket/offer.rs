use serde::{Deserialize, Serialize};

use super::SdpType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sdp {
    pub type_: SdpType,
    pub sdp: String,
}
