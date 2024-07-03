use std::net::SocketAddr;

use axum::extract::ws::{Message as WSMessage, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use loole::{Receiver, RecvError};
use tracing::{debug, error, info, warn};

use crate::Message;

pub async fn handle_socket(
    mut socket: WebSocket,
    socket_addr: SocketAddr,
    receiver: Receiver<Message>,
) {
    // send a ping (unsupported by some browsers) just to kick things off and get a
    // response
    if socket.send(WSMessage::Ping(vec![1, 2, 3])).await.is_ok() {
        debug!("WsClient {:?}: pinged", socket_addr);
    } else {
        error!("WsClient {:?}: could not send ping", socket_addr);
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine
        // anyway.
        return;
    }

    let (mut ws_sender, mut ws_receiver) = socket.split();
    loop {
        tokio::select! {
            // Receive any messages from the server/db/etc
            r = receiver.recv_async() => match r {
                Ok(m) => {
                    info!("WsClient {:?}: got message: {:?}", socket_addr, m);
                    // TODO: handle errors
                    // TODO: batch up sends
                    let _ = ws_sender.send(WSMessage::Text(format!("{:?}", m))).await;
                },
                Err(RecvError::Disconnected) => break,
            },

            // Receive messages from the websocket
            r = ws_receiver.next() => match r {
                Some(Ok(m)) => {
                    debug!("WsClient {:?}: got ws message: {:?}", socket_addr, m);
                    match m {
                        // WSMessage::Text(_) => todo!(),
                        // WSMessage::Binary(_) => todo!(),
                        // WSMessage::Ping(_) => todo!(),
                        // WSMessage::Pong(_) => todo!(),
                        WSMessage::Close(c) => {
                            if let Some(cf) = c {
                                debug!("WsClient {:?}: sent close with code {} and reason {}", socket_addr, cf.code, cf.reason);
                            } else {
                                warn!("WsClient {:?}: sent close without CloseFrame", socket_addr);
                            }
                            break;
                        },
                        // Already logged above
                        _ => {},
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

    // returning from the handler closes the websocket connection
    println!("Websocket context {socket_addr} destroyed");
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
