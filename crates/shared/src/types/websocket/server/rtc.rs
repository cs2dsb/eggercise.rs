use serde::{Deserialize, Serialize};

use super::ServerMessage;
use crate::types::{
    rtc::PeerId,
    websocket::{IceCandidate, Offer},
};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerRtc {
    /// Server allocated user id
    PeerId(PeerId),
    /// Room peers
    RoomPeers(Vec<PeerId>),
    /// An offer relayed from a peer
    PeerOffer { offer: Offer, peer: PeerId },
    /// An answer relayed from a peer
    PeerAnswer { answer: Offer, peer: PeerId },
    /// An ice candidate
    IceCandidate { candidate: IceCandidate, peer: PeerId },
}

impl From<ServerRtc> for ServerMessage {
    fn from(value: ServerRtc) -> Self {
        Self::Rtc(value)
    }
}

impl From<PeerId> for ServerMessage {
    fn from(value: PeerId) -> Self {
        Self::Rtc(ServerRtc::PeerId(value))
    }
}
