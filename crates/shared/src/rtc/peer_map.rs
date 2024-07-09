use dashmap::DashMap;
use just_webrtc::platform::PeerConnection;

pub type PeerMap = DashMap<u64, PeerConnection>;
