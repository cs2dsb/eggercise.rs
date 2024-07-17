use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SdpType {
    Offer,
    Pranswer,
    Answer,
    Rollback,
}

impl Into<web_sys::RtcSdpType> for SdpType {
    fn into(self) -> web_sys::RtcSdpType {
        use SdpType::*;
        match self {
            Offer => web_sys::RtcSdpType::Offer,
            Pranswer => web_sys::RtcSdpType::Pranswer,
            Answer => web_sys::RtcSdpType::Answer,
            Rollback => web_sys::RtcSdpType::Rollback,
        }
    }
}

impl From<web_sys::RtcSdpType> for SdpType {
    fn from(value: web_sys::RtcSdpType) -> Self {
        use SdpType::*;
        match value {
            web_sys::RtcSdpType::Pranswer => Pranswer,
            web_sys::RtcSdpType::Answer => Answer,
            web_sys::RtcSdpType::Rollback => Rollback,
            _ => Offer,
        }
    }
}
