use serde::{Deserialize, Serialize};

use super::ServerMessage;
use crate::types::{
    rtc::PeerId,
    websocket::{IceCandidate, Sdp},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomPeer {
    pub peer_id: PeerId,
    pub petname: String,
}

impl PartialEq for RoomPeer {
    fn eq(&self, other: &Self) -> bool {
        self.peer_id.eq(&other.peer_id)
    }
}

impl Eq for RoomPeer {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerRtc {
    /// Assign the peer an easy to read name mainly for making debug logs easier to read
    Petname(String),
    /// Room peers
    RoomPeers(Vec<RoomPeer>),
    /// An offer or answer relayed from a peer
    PeerSdp {
        sdp: Sdp,
        /// The peer_id of the peer the offer/answer is FROM
        /// Note: swapped from ClientRtc by the server
        peer_id: PeerId,
        /// The petname of the peer the offer/answer if FROM
        petname: String,
    },
    /// An ice candidate
    IceCandidate { candidate: IceCandidate, peer_id: PeerId },
}

impl From<ServerRtc> for ServerMessage {
    fn from(value: ServerRtc) -> Self {
        Self::Rtc(value)
    }
}
