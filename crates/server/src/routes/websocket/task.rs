#![allow(unused)]
use std::{net::SocketAddr, time::Duration};

use axum::extract::ws::{Message as WSMessage, WebSocket};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, StreamExt},
};
use loole::{Receiver, RecvError};
use petname::petname;
use shared::types::{
    rtc::{PeerId, RoomId},
    websocket::{ClientMessage, ClientRtc, RoomPeer, ServerMessage, ServerRtc, ServerUser},
};
use tokio::time;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, trace, warn};

use crate::{
    Client, ClientControlMessage, Clients, ClientsBySessionId, RtcRoomState, SessionId, UserState,
};

pub async fn handle_socket(
    socket: WebSocket,
    socket_addr: SocketAddr,
    user_state: Option<UserState>,
    receiver: Receiver<ClientControlMessage>,
    rtc_room_state: RtcRoomState,
    clients: Clients,
    session_id: SessionId,
    clients_by_session_id: ClientsBySessionId,
    client: Client,
) {
    if let Err(e) = handle_socket_inner(
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
    .await
    {
        error!("handle_socket error: {e:?}");
    }
}

async fn handle_socket_inner(
    mut socket: WebSocket,
    socket_addr: SocketAddr,
    mut user_state: Option<UserState>,
    receiver: Receiver<ClientControlMessage>,
    rtc_room_state: RtcRoomState,
    clients: Clients,
    session_id: SessionId,
    clients_by_session_id: ClientsBySessionId,
    client: Client,
) -> Result<(), anyhow::Error> {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    let login_state = if user_state.is_some() { ServerUser::Login } else { ServerUser::Logout };
    // Send out the login state
    ws_sender.send(login_state.try_into()?).await?;

    let petname = petname(2, "-").unwrap_or("<ran out of random names???>".into());
    let mut peer_id = None::<PeerId>;
    let mut interval = time::interval(Duration::from_secs(30));

    ws_sender.send(ServerMessage::from(ServerRtc::Petname(petname.clone())).try_into()?).await?;

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
                            debug!("{petname}: user logged in");
                            // Update the user that they are now logged in
                            ws_sender.send(ServerUser::Login.try_into()?).await;

                            // If they've given us their peer id
                            if let Some(peer_id) = peer_id.as_ref() {
                                // Join the user to the room
                                user_join_room(&mut ws_sender, &rtc_room_state, peer_id, &petname, user_state.as_ref().unwrap()).await?;
                            }
                        },
                        ClientControlMessage::Logout => {
                            let prev_user = user_state.take();
                            debug!("{petname}: user logged out");

                            // Update the user that they are now logged out
                            ws_sender.send(ServerUser::Logout.try_into()?).await;

                            // If they were logged in and have shared a peer id with us
                            if let (Some(user), Some(peer_id)) = (prev_user, peer_id.as_ref()) {
                                // Remove the user from the room
                                user_exit_room(&mut ws_sender, &rtc_room_state, peer_id, &user, &petname).await?;
                            }
                        },
                        ClientControlMessage::RtcStp { sdp, peer_id, petname } => {
                            let message: ServerMessage = ServerRtc::PeerSdp { sdp, peer_id, petname }.into();
                            ws_sender.send(message.try_into()?).await?;
                        },
                        ClientControlMessage::RtcIceCandidate { candidate, peer_id } => {
                            let message: ServerMessage = ServerRtc::IceCandidate { candidate, peer_id }.into();
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
                                debug!("{petname}: WsClient {socket_addr:?}: sent close with code {} and reason {}", cf.code, cf.reason);
                            } else {
                                warn!("{petname}: WsClient {socket_addr:?}: sent close without CloseFrame");
                            }
                            break;
                        },
                        text_or_binary => {
                            let message = ClientMessage::try_from(text_or_binary)?;
                            debug!("{petname}: WsClient {socket_addr:?}: got client message: {message:?}");
                            match message {
                                ClientMessage::Rtc(ClientRtc::Announce { peer_id: peer_id_ }) => {
                                    debug!("{petname}: Got peer id from client: {peer_id_:?}");
                                    peer_id = Some(peer_id_.clone());

                                    // Add the client to the client map now we have it's peer Id
                                    clients.add(peer_id_.clone(), client.clone());

                                    // Join the user room if logged in
                                    if let Some(user) = user_state.as_ref() {
                                        user_join_room(&mut ws_sender, &rtc_room_state, &peer_id_, &petname, user).await?;
                                    }
                                },
                                other => {
                                    // If they've given us their peer id
                                    if let Some(peer_id) = peer_id.as_ref() {
                                        handle_client_message(&mut ws_sender, peer_id, other, &clients, &socket_addr, &petname).await?;
                                    } else {
                                        error!("{petname}: Got client message: {other:?} but peer id wasn't set");
                                    }
                                },
                            }
                        },
                    }
                },
                Some(Err(e)) => {
                    error!("{petname}: WsClient {:?}: recv error: {:?}", socket_addr, e);
                    break;
                },
                None => {
                    warn!("{petname}: WsClient {:?}: got None before Close", socket_addr);
                    break;
                },
            },
        }
    }

    if let (Some(user), Some(peer_id)) = (user_state.as_ref(), peer_id.as_ref()) {
        // Remove the user from the room
        user_exit_room(&mut ws_sender, &rtc_room_state, peer_id, user, &petname).await?;
    }

    if let Some(peer_id) = peer_id.as_ref() {
        clients.remove(peer_id);
    }

    clients_by_session_id.remove(&session_id, &socket_addr);

    // returning from the handler closes the websocket connection
    debug!("{petname}: context destroyed ({socket_addr})");
    trace!("Clients: {clients:?}");
    trace!("Rooms: {rtc_room_state:?}");

    Ok(())
}

async fn user_join_room(
    ws_sender: &mut SplitSink<WebSocket, WSMessage>,
    rtc_room_state: &RtcRoomState,
    peer_id: &PeerId,
    petname: &str,
    user: &UserState,
) -> Result<(), anyhow::Error> {
    let room_peer = RoomPeer { peer_id: peer_id.to_owned(), petname: petname.to_string() };

    let room_id = RoomId::from(**user.id);
    rtc_room_state.add(room_id.clone(), room_peer);

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
    petname: &str,
) -> Result<(), anyhow::Error> {
    let room_id = RoomId::from(**user.id);
    debug!("{petname}: exiting room");
    rtc_room_state.remove(room_id, peer_id.clone());
    Ok(())
}

async fn handle_client_message(
    ws_sender: &mut SplitSink<WebSocket, WSMessage>,
    peer_id: &PeerId,
    message: ClientMessage,
    clients: &Clients,
    socket_addr: &SocketAddr,
    petname: &str,
) -> Result<(), anyhow::Error> {
    match message {
        ClientMessage::Rtc(ClientRtc::Sdp { sdp, peer_id: target_peer_id, petname }) => {
            if let Some(peer_client) = clients.get(&target_peer_id) {
                debug!(
                    "{petname}: Forwarding sdp (type: {:?}) from {peer_id} to {target_peer_id}",
                    sdp.type_
                );
                peer_client
                    .send(ClientControlMessage::RtcStp { sdp, peer_id: peer_id.clone(), petname })
                    .await?;
            } else {
                error!("{petname}: Failed to find client matching offer peer_id {target_peer_id}");
                // TODO: send error back to client
            }
        },
        ClientMessage::Rtc(ClientRtc::IceCandidate { candidate, peer_id: target_peer_id }) => {
            if let Some(peer_client) = clients.get(&target_peer_id) {
                debug!("{petname}: Forwarding ice candidate from {peer_id} to {target_peer_id}");
                peer_client
                    .send(ClientControlMessage::RtcIceCandidate {
                        candidate,
                        peer_id: peer_id.clone(),
                    })
                    .await?;
            } else {
                error!(
                    "{petname}: Failed to find client matching ice candidate peer_id \
                     {target_peer_id}"
                );
                // TODO: send error back to client
            }
        },

        // Handled by outer loop
        ClientMessage::Rtc(ClientRtc::Announce { .. }) => unreachable!(),
    }

    Ok(())
}
