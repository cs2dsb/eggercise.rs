use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use web_sys::{RtcIceCandidate, RtcIceCandidateInit};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_m_line_index: Option<u16>,
}

#[cfg(feature = "wasm")]
impl From<RtcIceCandidate> for IceCandidate {
    fn from(value: RtcIceCandidate) -> Self {
        Self {
            candidate: value.candidate(),
            sdp_mid: value.sdp_mid(),
            sdp_m_line_index: value.sdp_m_line_index(),
        }
    }
}

#[cfg(feature = "wasm")]
impl From<IceCandidate> for RtcIceCandidateInit {
    fn from(value: IceCandidate) -> Self {
        let mut init = RtcIceCandidateInit::new(&value.candidate);

        init.sdp_mid(value.sdp_mid.as_deref());
        init.sdp_m_line_index(value.sdp_m_line_index);

        init
    }
}
