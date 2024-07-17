use std::net::{IpAddr, SocketAddr};

use axum::{
    extract::{connect_info::ConnectInfo, ws::WebSocketUpgrade},
    http::HeaderMap,
    response::IntoResponse,
};
use tracing::debug;

use crate::{
    constants::WEBSOCKET_CHANNEL_BOUND, routes::websocket::handle_socket, Client, Clients,
    PeerIdState, RtcRoomState, UserState,
};

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    // We want the address as a key for the client map
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    // If X-Forwarded-* is set use it to override the addr
    headers: HeaderMap,
    // Optional user
    user_state: Option<UserState>,
    // Peer id allocated as part of session state
    peer_id: PeerIdState,
    // Map containing all connected clients
    clients: Clients,
    rtc_room_state: RtcRoomState,
) -> impl IntoResponse {
    debug!("Websocket upgrade headers: {:?}", headers);

    let ip = headers
        .get("x-forwarded-for")
        .map(|v| v.to_str().ok())
        .flatten()
        .map(|v| v.parse::<IpAddr>().ok())
        .flatten()
        .unwrap_or(addr.ip());

    let port = headers
        .get("x-forwarded-port")
        .map(|v| v.to_str().ok())
        .flatten()
        .map(|v| v.parse::<u16>().ok())
        .flatten()
        .unwrap_or(addr.port());

    let socket_addr = SocketAddr::new(ip, port);

    let (sender, receiver) = loole::bounded(WEBSOCKET_CHANNEL_BOUND);

    let client = Client::new((*peer_id).clone(), sender);

    if let Some(old_client) = clients.add(client) {
        debug!("A client with the same key evicted a previous client: {:?}", old_client);
    }

    // Complete the upgrade to a websocket
    ws.on_upgrade(move |socket| {
        handle_socket(socket, socket_addr, user_state, peer_id, receiver, rtc_room_state, clients)
    })
}
