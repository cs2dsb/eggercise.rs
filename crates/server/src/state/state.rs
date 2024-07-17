use std::sync::Arc;

use deadpool_sqlite::Pool;

use super::rtc::RtcRoomState;
use crate::{
    cli::Cli,
    state::{VapidPrivateKey, VapidPubKey},
    Clients,
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: Pool,
    pub webauthn: Arc<webauthn_rs::Webauthn>,
    pub args: Arc<Cli>,
    pub vapid_pub_key: VapidPubKey,
    pub vapid_private_key: VapidPrivateKey,
    pub websocket_clients: Clients,
    pub rtc_room_state: RtcRoomState,
}
