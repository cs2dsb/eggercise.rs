use std::net::{IpAddr, SocketAddr};

use axum::{
    extract::{connect_info::ConnectInfo, ws::WebSocketUpgrade},
    http::HeaderMap,
    response::IntoResponse,
};
use tracing::{debug, warn};

use super::handle_socket;
use crate::{constants::WEBSOCKET_CHANNEL_BOUND, Client, Clients, UserState};

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    // We want the address as a key for the client map
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    // If X-Forwarded-* is set use it to override the addr
    headers: HeaderMap,
    // User has to be logged in and we need their ID
    user_state: UserState,
    // Map containing all connected clients
    clients: Clients,
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

    let client = Client::new(user_state.id, socket_addr.clone(), sender);

    if let Some(old_client) = clients.add(client) {
        warn!(
            "A client with the same user_id & socket address evicted a previous client: {:?}",
            old_client
        );
    }

    // Complete the upgrade to a websocket
    ws.on_upgrade(move |socket| handle_socket(socket, socket_addr, receiver))
}
