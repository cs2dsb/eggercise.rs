use serde::{Deserialize, Serialize};

use crate::api::error::Nothing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RtcSdpType {
    Offer,
    Pranswer,
    Answer,
    Rollback,
}

impl Into<web_sys::RtcSdpType> for RtcSdpType {
    fn into(self) -> web_sys::RtcSdpType {
        use RtcSdpType::*;
        match self {
            Offer => web_sys::RtcSdpType::Offer,
            Pranswer => web_sys::RtcSdpType::Pranswer,
            Answer => web_sys::RtcSdpType::Answer,
            Rollback => web_sys::RtcSdpType::Rollback,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtcOfferRequest {
    pub sdp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtcOfferResponse {
    pub type_: RtcSdpType,
    pub sdp: String,
    pub candidate: String,
}

pub type RtcOfferError = Nothing;

// response_error!(RtcOfferError {});
