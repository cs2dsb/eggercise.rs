#![allow(unused)]
use std::{
    any::type_name,
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use dashmap::DashMap;
use futures::{channel::mpsc::UnboundedReceiver, SinkExt, StreamExt};
use gloo::{net::websocket, utils::errors::JsError};
#[cfg(feature = "debug-signals")]
use leptos::{create_rw_signal, RwSignal, Signal};
use leptos::{provide_context, spawn_local, store_value, use_context, SignalUpdate, StoredValue};
use reconnecting_websocket::SocketSink;
use shared::{
    api::{
        error::{FrontendError, Nothing},
        Object,
    },
    rtc::{
        peer_connector::Connector as _,
        signalling_client::{self, Client as _},
        Builder as _, PeerConnector, PeerMap, SignallingClient,
    },
    types::{
        rtc::{PeerId, RoomId},
        websocket::{ClientMessage, ClientRtc, Offer, ServerRtc},
    },
};
use tracing::{debug, error, info};
use wasm_bindgen::{convert::FromWasmAbi, prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::Function, ErrorEvent, Event, MessageEvent, RtcAnswerOptions, RtcConfiguration,
    RtcDataChannel, RtcDataChannelEvent, RtcDataChannelInit, RtcDataChannelType,
    RtcIceCandidateInit, RtcOfferOptions, RtcPeerConnection, RtcPeerConnectionIceEvent, RtcSdpType,
    RtcSessionDescription, RtcSessionDescriptionInit,
};

use crate::{
    api::send_offer,
    utils::{
        location::{host, protocol},
        websocket::Websocket,
    },
};

struct Peer {
    waker: Rc<RefCell<Option<Waker>>>,
    channel: RtcDataChannel,
    peer: RtcPeerConnection,
    their_peer_id: PeerId,

    // Hang on to them for de-registering
    closures: (
        Closure<dyn FnMut(ErrorEvent)>,
        Closure<dyn FnMut(MessageEvent)>,
        Closure<dyn FnMut(RtcDataChannelEvent)>,
        Closure<dyn FnMut(RtcPeerConnectionIceEvent)>,
        Closure<dyn FnMut(Event)>,
        Closure<dyn FnMut(Event)>,
        Closure<dyn FnMut(Event)>,
    ),
}

impl Peer {
    async fn new(
        waker: Rc<RefCell<Option<Waker>>>,
        our_peer_id: PeerId,
        their_peer_id: PeerId,
        sender: &mut SocketSink<ClientMessage>,
        offer: Option<Offer>,
    ) -> Result<Self, FrontendError<Nothing>> {
        let peer = {
            let mut config = RtcConfiguration::new();
            config.peer_identity(Some(&our_peer_id.to_string()));

            RtcPeerConnection::new_with_configuration(&config)?
        };

        let channel = {
            let mut config = RtcDataChannelInit::new();
            config.ordered(true);

            let mut channel =
                peer.create_data_channel_with_data_channel_dict("client data", &config);

            channel.set_binary_type(RtcDataChannelType::Arraybuffer);

            channel
        };

        let channel_error_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            // TODO: Not sure this is the right event type
            Closure::wrap(Box::new(move |event: ErrorEvent| {
                debug!("channel_error callback: {event:?}");
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        channel.add_event_listener_with_callback(
            "error",
            channel_error_callback.as_ref().unchecked_ref(),
        )?;

        let channel_message_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |event: MessageEvent| {
                debug!("channel_message callback: {event:?}");
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        channel.add_event_listener_with_callback(
            "message",
            channel_message_callback.as_ref().unchecked_ref(),
        )?;

        let channel_open_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |event: RtcDataChannelEvent| {
                debug!("channel_open callback: {event:?}");
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        channel.add_event_listener_with_callback(
            "open",
            channel_open_callback.as_ref().unchecked_ref(),
        )?;

        let peer_icecandidate_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
                debug!("peer_icecandidate callback: {event:?}");
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        peer.add_event_listener_with_callback(
            "icecandidate",
            peer_icecandidate_callback.as_ref().unchecked_ref(),
        )?;

        let peer_iceconnectionstatechange_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |event: Event| {
                debug!("peer_iceconnectionstatechange callback: {event:?}");
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        peer.add_event_listener_with_callback(
            "iceconnectionstatechange",
            peer_iceconnectionstatechange_callback.as_ref().unchecked_ref(),
        )?;

        let peer_connectionstatechange_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |event: Event| {
                debug!("peer_connectionstatechange callback: {event:?}");
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        peer.add_event_listener_with_callback(
            "connectionstatechange",
            peer_connectionstatechange_callback.as_ref().unchecked_ref(),
        )?;

        let peer_icegatheringstatechange_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |event: Event| {
                debug!("peer_icegatheringstatechange callback: {event:?}");
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        peer.add_event_listener_with_callback(
            "icegatheringstatechange",
            peer_icegatheringstatechange_callback.as_ref().unchecked_ref(),
        )?;

        if let Some(offer) = offer {
            let mut offer_desc = RtcSessionDescriptionInit::new(offer.type_.into());
            offer_desc.sdp(&offer.sdp);

            JsFuture::from(peer.set_remote_description(&offer_desc)).await?;

            let answer_init: RtcSessionDescriptionInit =
                JsFuture::from(peer.create_answer()).await?.into();

            JsFuture::from(peer.set_local_description(&answer_init)).await?;

            let answer_desc = peer.local_description().ok_or(FrontendError::Client {
                message: "Local description missing after it was set".to_string(),
            })?;

            let type_ = answer_desc.type_();
            let sdp = answer_desc.sdp();

            let answer = Offer { type_: type_.into(), sdp };

            sender.send(ClientRtc::Answer { answer, peer: their_peer_id.clone() }.into()).await?;
        } else {
            let offer_init: RtcSessionDescriptionInit =
                JsFuture::from(peer.create_offer()).await?.into();

            JsFuture::from(peer.set_local_description(&offer_init)).await?;

            let offer_desc = peer.local_description().ok_or(FrontendError::Client {
                message: "Local description missing after it was set".to_string(),
            })?;

            let type_ = offer_desc.type_();
            let sdp = offer_desc.sdp();

            let offer = Offer { type_: type_.into(), sdp };

            sender.send(ClientRtc::Offer { offer, peer: their_peer_id.clone() }.into()).await?;
        }

        // let response = send_offer(sdp).await.map_err(|e| FrontendError::Client {
        //     message: format!("RTC offer response error: {e:?}"),
        // })?;

        // info!("Got rtc offer response: {:?}", response);
        // let answer = {
        //     let mut answer = RtcSessionDescriptionInit::new(response.type_.into());
        //     answer.sdp(&response.sdp);
        //     answer
        // };
        // JsFuture::from(peer.set_remote_description(&answer)).await?;
        // info!("Remote description set successfully");

        // let candidate: RtcIceCandidateInit = { todo!() };
        // JsFuture::from(peer.add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(&candidate)))
        //     .await?;
        // info!("Ice candidate added successfully");

        let closures = (
            channel_error_callback,
            channel_message_callback,
            channel_open_callback,
            peer_icecandidate_callback,
            peer_iceconnectionstatechange_callback,
            peer_connectionstatechange_callback,
            peer_icegatheringstatechange_callback,
        );

        Ok(Self { waker, closures, peer, channel, their_peer_id })
    }

    async fn answer(
        &mut self,
        sender: &mut SocketSink<ClientMessage>,
        answer: Offer,
    ) -> Result<(), FrontendError<Nothing>> {
        Ok(())
    }

    fn close(self) {
        for (event, closure) in [
            ("error", self.closures.0.as_ref()),
            ("message", self.closures.1.as_ref()),
            ("open", self.closures.2.as_ref()),
            ("icecandidate", self.closures.3.as_ref()),
        ] {
            let _ =
                self.channel.remove_event_listener_with_callback(&event, closure.unchecked_ref());
        }

        for (event, closure) in [
            ("iceconnectionstatechange", self.closures.4.as_ref()),
            ("connectionstatechange", self.closures.5.as_ref()),
            ("icegatheringstatechange", self.closures.6.as_ref()),
        ] {
            let _ = self.peer.remove_event_listener_with_callback(&event, closure.unchecked_ref());
        }
    }
}

pub struct RtcSource {
    receiver: UnboundedReceiver<ServerRtc>,
}

impl From<UnboundedReceiver<ServerRtc>> for RtcSource {
    fn from(receiver: UnboundedReceiver<ServerRtc>) -> Self {
        Self { receiver }
    }
}

struct RtcInner {
    waker: Rc<RefCell<Option<Waker>>>,
    peers: Arc<DashMap<PeerId, Peer>>,

    #[cfg(feature = "debug-signals")]
    peers_signal: RwSignal<Vec<String>>,
}

impl RtcInner {
    fn new(
        mut source: RtcSource,
        mut sender: SocketSink<ClientMessage>,
        waker: Rc<RefCell<Option<Waker>>>,

        #[cfg(feature = "debug-signals")] peers_signal: RwSignal<Vec<String>>,
    ) -> Self {
        let peers: Arc<DashMap<PeerId, Peer>> = Default::default();

        let waker_ = Rc::clone(&waker);
        let peers_ = peers.clone();
        let peers_signal_ = peers_signal.clone();
        spawn_local(async move {
            let r = async move {
                let mut peer_id = None;

                loop {
                    peers_signal
                        .update(|v| v.push(format!("Peer count: {}", peers_.iter().count())));

                    if let Some(m) = source.receiver.next().await {
                        match m {
                            ServerRtc::PeerId(p) => {
                                info!("got peer id: {p:?}");
                                peer_id = Some(p);
                            },
                            ServerRtc::RoomPeers(peers) => {
                                info!("got peers: {peers:?}");
                                let peer_id = peer_id.as_ref().ok_or(FrontendError::Other {
                                    message: format!("Got room peers before peer id was set"),
                                })?;

                                for their_peer_id in peers {
                                    if peers_.contains_key(&their_peer_id) {
                                        continue;
                                    }

                                    peers_signal.update(|v| {
                                        v.push(format!("RoomPeers offering: {}", their_peer_id))
                                    });
                                    let peer = Peer::new(
                                        Rc::clone(&waker_),
                                        peer_id.clone(),
                                        their_peer_id.clone(),
                                        &mut sender,
                                        None,
                                    )
                                    .await?;

                                    peers_.insert(their_peer_id, peer);
                                }
                            },

                            ServerRtc::PeerOffer { offer, peer } => {
                                info!("got peer offer: {peer:?}, {offer:?}");
                                let peer_id = peer_id.as_ref().ok_or(FrontendError::Other {
                                    message: format!("Got room peers before peer id was set"),
                                })?;

                                let their_peer_id = peer;

                                peers_signal.update(|v| {
                                    v.push(format!("PeerOffer answering: {}", their_peer_id))
                                });
                                let peer = Peer::new(
                                    Rc::clone(&waker_),
                                    peer_id.clone(),
                                    their_peer_id.clone(),
                                    &mut sender,
                                    Some(offer),
                                )
                                .await?;

                                peers_.insert(their_peer_id, peer);
                            },

                            ServerRtc::PeerAnswer { answer, peer } => {
                                info!("got peer answer: {peer:?}, {answer:?}");
                                if let Some(mut peer) = peers_.get_mut(&peer) {
                                    peers_signal.update(|v| {
                                        v.push(format!("PeerAnswer updating: {}", peer.key()))
                                    });
                                    peer.value_mut().answer(&mut sender, answer).await?;
                                }
                            },
                        }
                    } else {
                        info!("source closed");
                        break;
                    }
                }
                Ok::<_, FrontendError<Nothing>>(())
            }
            .await;

            if let Err(e) = r {
                error!("Error from rtc inner task: {e:?}");
            }
        });

        Self {
            waker,
            peers,
            #[cfg(feature = "debug-signals")]
            peers_signal,
        }
    }

    fn close(self) {
        let keys = self.peers.iter().map(|r| r.key().clone()).collect::<Vec<_>>();
        for k in keys.into_iter() {
            if let Some((_, v)) = self.peers.remove(&k) {
                v.close();
            }
        }
    }
}

#[derive(Clone)]
pub struct Rtc {
    inner: StoredValue<RtcInner>,

    #[cfg(feature = "debug-signals")]
    peers_signal: RwSignal<Vec<String>>,
}

impl Rtc {
    async fn new(
        source: RtcSource,
        sender: SocketSink<ClientMessage>,
    ) -> Result<Self, FrontendError<Nothing>> {
        assert!(
            use_context::<Self>().is_none(),
            "Call to Rtc::new() when there's already one in the context"
        );

        let waker: Rc<RefCell<Option<Waker>>> = Default::default();

        #[cfg(feature = "debug-signals")]
        let peers_signal = create_rw_signal(Vec::new());

        let inner = RtcInner::new(
            source,
            sender,
            waker,
            #[cfg(feature = "debug-signals")]
            peers_signal,
        );

        let inner = store_value(inner);

        Ok(Self {
            inner,
            #[cfg(feature = "debug-signals")]
            peers_signal,
        })
    }

    pub async fn provide_context(
        source: RtcSource,
        sender: SocketSink<ClientMessage>,
    ) -> Result<(), FrontendError<Nothing>> {
        if use_context::<Self>().is_none() {
            let v = Self::new(source, sender).await?;
            provide_context(v);
        }
        Ok(())
    }

    pub fn use_rtc() -> Self {
        use_context::<Self>().expect(&format!("{} missing from context", type_name::<Self>()))
    }

    #[cfg(feature = "debug-signals")]
    pub fn peers_signal() -> Signal<Vec<String>> {
        Self::use_rtc().peers_signal.into()
    }
}
