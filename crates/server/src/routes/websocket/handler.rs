use std::net::{IpAddr, SocketAddr};

use axum::{
    extract::{connect_info::ConnectInfo, ws::WebSocketUpgrade},
    http::HeaderMap,
    response::IntoResponse,
};
use tracing::debug;

use crate::{
    constants::WEBSOCKET_CHANNEL_BOUND, routes::websocket::handle_socket, Client, Clients,
    ClientsBySessionId, RtcRoomState, SessionId, UserState,
};

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    // We want the address as a key for the client map
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    // If X-Forwarded-* is set use it to override the addr
    headers: HeaderMap,
    // Optional user
    user_state: Option<UserState>,
    // Map containing all connected clients by socket
    clients: Clients,
    // Map containing all connected clients by session id
    clients_by_session_id: ClientsBySessionId,
    rtc_room_state: RtcRoomState,
    session_id: SessionId,
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

    let client = Client::new(socket_addr.clone(), sender);

    clients_by_session_id.add(session_id.clone(), client.clone());

    // Complete the upgrade to a websocket
    ws.on_upgrade(move |socket| {
        handle_socket(
            socket,
            socket_addr,
            user_state,
            receiver,
            rtc_room_state,
            clients,
            session_id,
            clients_by_session_id,
            client,
        )
    })
}
