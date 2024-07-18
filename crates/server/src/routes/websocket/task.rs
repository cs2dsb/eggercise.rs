#![allow(unused)]
use std::{net::SocketAddr, time::Duration};

use axum::extract::ws::{Message as WSMessage, WebSocket};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, StreamExt},
};
use loole::{Receiver, RecvError};
use shared::types::{
    rtc::{PeerId, RoomId},
    websocket::{ClientMessage, ClientRtc, ServerMessage, ServerRtc, ServerUser},
};
use tokio::time;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

use crate::{ClientControlMessage, Clients, PeerIdState, RtcRoomState, UserState};

pub async fn handle_socket(
    socket: WebSocket,
    socket_addr: SocketAddr,
    user_state: Option<UserState>,
    peer_id: PeerIdState,
    receiver: Receiver<ClientControlMessage>,
    rtc_room_state: RtcRoomState,
    clients: Clients,
) {
    if let Err(e) = handle_socket_inner(
        socket,
        socket_addr,
        user_state,
        peer_id,
        receiver,
        rtc_room_state,
        clients,
    )
    .await
    {
        error!("handle_socket error: {e:?}");
    }
}

async fn handle_socket_inner(
    mut socket: WebSocket,
    socket_addr: SocketAddr,
    mut user_state: Option<UserState>,
    peer_id: PeerIdState,
    receiver: Receiver<ClientControlMessage>,
    rtc_room_state: RtcRoomState,
    clients: Clients,
) -> Result<(), anyhow::Error> {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    let peer_id: PeerId = (*peer_id).clone();
    let peer_id_msg: ServerMessage = peer_id.clone().into();

    let login_state = if user_state.is_some() { ServerUser::Login } else { ServerUser::Logout };
    // Send out the login state
    ws_sender.send(login_state.try_into()?).await?;
    // Send out the session specific peer id
    ws_sender.send(peer_id_msg.try_into()?).await?;

    // Join the user room if logged in
    if let Some(user) = user_state.as_ref() {
        user_join_room(&mut ws_sender, &rtc_room_state, &peer_id, user).await?;
    }

    let mut interval = time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                ws_sender.send(WSMessage::Ping(vec![])).await?;
            },
            // Receive any messages from the server/db/etc
            r = receiver.recv_async() => match r {
                Ok(m) => {
                    match m {
                        ClientControlMessage::Login(user_state_) => {
                            user_state = Some(user_state_);
                            debug!("user logged in");
                            // Update the user that they are now logged in
                            ws_sender.send(ServerUser::Login.try_into()?).await;

                            // Join the user to the room
                            user_join_room(&mut ws_sender, &rtc_room_state, &peer_id, user_state.as_ref().unwrap()).await?;
                        },
                        ClientControlMessage::Logout => {
                            let prev_user = user_state.take();
                            debug!("user logged out");

                            // Update the user that they are now logged out
                            ws_sender.send(ServerUser::Logout.try_into()?).await;

                            if let Some(user) = prev_user {
                                // Remove the user from the room
                                user_exit_room(&mut ws_sender, &rtc_room_state, &peer_id, &user).await?;
                            }
                        },
                        ClientControlMessage::RtcOffer { offer, peer } => {
                            let message: ServerMessage = ServerRtc::PeerOffer { offer, peer }.into();
                            ws_sender.send(message.try_into()?).await?;
                        },
                        ClientControlMessage::RtcAnswer { answer, peer } => {
                            let message: ServerMessage = ServerRtc::PeerAnswer { answer, peer }.into();
                            ws_sender.send(message.try_into()?).await?;
                        },
                        ClientControlMessage::RtcIceCandidate { candidate, peer } => {
                            let message: ServerMessage = ServerRtc::IceCandidate { candidate, peer }.into();
                            ws_sender.send(message.try_into()?).await?;
                        }
                    }
                },
                Err(RecvError::Disconnected) => break,
            },

            // Receive messages from the websocket
            r = ws_receiver.next() => match r {
                Some(Ok(m)) => {
                    match m {
                        WSMessage::Ping(_) => debug!("WSClient {socket_addr:?} ping"),
                        WSMessage::Pong(_) => debug!("WSClient {socket_addr:?} pong"),
                        WSMessage::Close(c) => {
                            if let Some(cf) = c {
                                debug!("WsClient {socket_addr:?}: sent close with code {} and reason {}", cf.code, cf.reason);
                            } else {
                                warn!("WsClient {socket_addr:?}: sent close without CloseFrame");
                            }
                            break;
                        },
                        text_or_binary => {
                            let message = ClientMessage::try_from(text_or_binary)?;
                            debug!("WsClient {socket_addr:?}: got client message: {message:?}");
                            handle_client_message(&mut ws_sender, &peer_id, message, &clients).await?;
                        },
                    }
                },
                Some(Err(e)) => {
                    error!("WsClient {:?}: recv error: {:?}", socket_addr, e);
                    break;
                },
                None => {
                    warn!("WsClient {:?}: got None before Close", socket_addr);
                    break;
                },
            },
        }
    }

    clients.remove(&peer_id);

    // returning from the handler closes the websocket connection
    debug!("context destroyed ({socket_addr})");

    Ok(())
}

async fn user_join_room(
    ws_sender: &mut SplitSink<WebSocket, WSMessage>,
    rtc_room_state: &RtcRoomState,
    peer_id: &PeerId,
    user: &UserState,
) -> Result<(), anyhow::Error> {
    let room_id = RoomId::from(**user.id);
    rtc_room_state.add(room_id.clone(), peer_id.clone());

    let peers = rtc_room_state.room_peers(&room_id, peer_id);
    if peers.len() > 0 {
        let message: ServerMessage = ServerRtc::RoomPeers(peers).into();
        ws_sender.send(message.try_into()?).await?;
    }

    Ok(())
}

async fn user_exit_room(
    ws_sender: &mut SplitSink<WebSocket, WSMessage>,
    rtc_room_state: &RtcRoomState,
    peer_id: &PeerId,
    user: &UserState,
) -> Result<(), anyhow::Error> {
    let room_id = RoomId::from(**user.id);
    rtc_room_state.remove(room_id, peer_id.clone());
    Ok(())
}

async fn handle_client_message(
    ws_sender: &mut SplitSink<WebSocket, WSMessage>,
    peer_id: &PeerId,
    message: ClientMessage,
    clients: &Clients,
) -> Result<(), anyhow::Error> {
    match message {
        ClientMessage::Rtc(ClientRtc::Offer { offer, peer }) => {
            if let Some(peer_client) = clients.get(&peer) {
                debug!("Forwarding offer from {peer_id} to {peer}");
                peer_client
                    .send(ClientControlMessage::RtcOffer { offer, peer: peer_id.clone() })
                    .await?;
            } else {
                error!("Failed to find client matching offer peer_id {peer}");
                // TODO: send error back to client
            }
        },
        ClientMessage::Rtc(ClientRtc::Answer { answer, peer }) => {
            if let Some(peer_client) = clients.get(&peer) {
                debug!("Forwarding answer from {peer_id} to {peer}");
                peer_client
                    .send(ClientControlMessage::RtcAnswer { answer, peer: peer_id.clone() })
                    .await?;
            } else {
                error!("Failed to find client matching answer peer_id {peer}");
                // TODO: send error back to client
            }
        },
        ClientMessage::Rtc(ClientRtc::IceCandidate { candidate, peer }) => {
            if let Some(peer_client) = clients.get(&peer) {
                debug!("Forwarding ice candidate from {peer_id} to {peer}");
                peer_client
                    .send(ClientControlMessage::RtcIceCandidate {
                        candidate,
                        peer: peer_id.clone(),
                    })
                    .await?;
            } else {
                error!("Failed to find client matching ice candidate peer_id {peer}");
                // TODO: send error back to client
            }
        },
    }

    Ok(())
}

// /// helper to print contents of messages to stdout. Has special treatment for
// /// Close.
// fn process_message(msg: WSMessage, who: SocketAddr) -> ControlFlow<(), ()> {
//     match msg {
//         WSMessage::Text(t) => {
//             println!(">>> {who} sent str: {t:?}");
//         }
//         WSMessage::Binary(d) => {
//             println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
//         }
//         WSMessage::Close(c) => {
//             if let Some(cf) = c {
//                 println!(
//                     ">>> {} sent close with code {} and reason `{}`",
//                     who, cf.code, cf.reason
//                 );
//             } else {
//                 println!(">>> {who} somehow sent close message without
// CloseFrame");             }
//             return ControlFlow::Break(());
//         }

//         WSMessage::Pong(v) => {
//             println!(">>> {who} sent pong with {v:?}");
//         }
//         // You should never need to manually handle WSMessage::Ping, as
// axum's websocket library         // will do so for you automagically by
// replying with Pong and copying the v according to         // spec. But if you
// need the contents of the pings you can see them here.
//         WSMessage::Ping(v) => {
//             println!(">>> {who} sent ping with {v:?}");
//         }
//     }
//     ControlFlow::Continue(())
// }
