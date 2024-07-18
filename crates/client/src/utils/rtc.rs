#![allow(unused)]
use std::{
    any::type_name,
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use dashmap::{mapref::one::RefMut, DashMap};
use futures::{channel::mpsc::UnboundedReceiver, SinkExt, StreamExt};
use gloo::{net::websocket, timers::future::IntervalStream, utils::errors::JsError};
#[cfg(feature = "debug-signals")]
use leptos::{create_rw_signal, RwSignal, Signal};
use leptos::{provide_context, spawn_local, store_value, use_context, SignalUpdate, StoredValue};
use reconnecting_websocket::SocketSink;
use shared::{
    api::{
        error::{FrontendError, Nothing, ResultContext},
        Object,
    },
    rtc::{
        peer_connector::Connector as _,
        signalling_client::{self, Client as _},
        Builder as _, PeerConnector, PeerMap, SignallingClient,
    },
    types::{
        rtc::{PeerId, RoomId},
        websocket::{ClientMessage, ClientRtc, IceCandidate, Offer, ServerRtc},
    },
};
use tracing::{debug, error, info, warn};
use wasm_bindgen::{convert::FromWasmAbi, prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::{Function, Reflect},
    ErrorEvent, Event, MessageEvent, RtcAnswerOptions, RtcConfiguration, RtcDataChannel,
    RtcDataChannelEvent, RtcDataChannelInit, RtcDataChannelType, RtcIceCandidate,
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

/// The role this peer is taking
/// See <https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API/Perfect_negotiation>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PerfectRole {
    Polite,
    Impolite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OfferState {
    None,
    Offered,
    Answered,
    Accepted,
}

struct Peer {
    waker: Rc<RefCell<Option<Waker>>>,
    signalling_sender: SocketSink<ClientMessage>,
    channel: Option<RtcDataChannel>,
    peer: RtcPeerConnection,
    our_peer_id: PeerId,
    their_peer_id: PeerId,
    role: PerfectRole,
    offer_state: OfferState,

    // Hang on to them for de-registering
    closures: (
        Option<Closure<dyn FnMut(ErrorEvent)>>,
        Option<Closure<dyn FnMut(MessageEvent)>>,
        Option<Closure<dyn FnMut(RtcDataChannelEvent)>>,
        Option<Closure<dyn FnMut(Event)>>,
        Closure<dyn FnMut(RtcPeerConnectionIceEvent)>,
        Closure<dyn FnMut(Event)>,
        Closure<dyn FnMut(Event)>,
        Closure<dyn FnMut(Event)>,
        Closure<dyn FnMut(RtcDataChannelEvent)>,
    ),
}

impl Peer {
    fn new(
        waker: Rc<RefCell<Option<Waker>>>,
        signalling_sender: SocketSink<ClientMessage>,
        our_peer_id: PeerId,
        their_peer_id: PeerId,
    ) -> Result<Self, FrontendError<Nothing>> {
        // Since both parties know the peer IDs this will result in them picking the correct roles
        let role =
            if our_peer_id > their_peer_id { PerfectRole::Polite } else { PerfectRole::Impolite };

        let offer_state = OfferState::None;

        let peer = {
            let mut config = RtcConfiguration::new();

            // TODO: peer identity is actually for the remote peer and requires validation
            // config.peer_identity(Some(&our_peer_id.to_string()));

            RtcPeerConnection::new_with_configuration(&config)?
        };

        let signalling_sender_ = signalling_sender.clone();
        let their_peer_id_ = their_peer_id.clone();
        let peer_icecandidate_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
                if let Some(candidate) = event.candidate() {
                    debug!("peer_icecandidate callback: {}", candidate.candidate());
                    let mut signalling_sender_ = signalling_sender_.clone();
                    let their_peer_id_ = their_peer_id_.clone();
                    // TODO: might be preferable to have a channel with a sync send to send this
                    // back to the main loop?
                    spawn_local(async move {
                        if let Err(e) = signalling_sender_
                            .send(
                                ClientRtc::IceCandidate {
                                    candidate: candidate.into(),
                                    peer: their_peer_id_.clone(),
                                }
                                .into(),
                            )
                            .await
                        {
                            error!("Error sending ice candidate to signalling server: {e:?}");
                        }
                    });
                } else {
                    debug!("peer_icecandidate callback: Null candidiate");
                }
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
            let target_key = JsValue::from_str("target");
            Closure::wrap(Box::new(move |event: Event| {
                let peer: RtcPeerConnection =
                    Reflect::get(&event, &target_key).expect("event.connection").into();

                debug!("peer_icegatheringstatechange callback: {:?}", peer.ice_gathering_state());
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        peer.add_event_listener_with_callback(
            "icegatheringstatechange",
            peer_icegatheringstatechange_callback.as_ref().unchecked_ref(),
        )?;

        let peer_datachannel_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            let channel_key = JsValue::from_str("channel");
            Closure::wrap(Box::new(move |event: RtcDataChannelEvent| {
                let channel = event.channel();

                debug!("peer_datachannel callback: {:?}", channel);

                spawn_local(async move {
                    let mut stream = IntervalStream::new(500).fuse();
                    let mut i = 0_usize;
                    loop {
                        let _ = stream.next().await;

                        channel.send_with_str(&format!("Hello {i}")).unwrap();
                        i += 1;
                    }
                });
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        peer.add_event_listener_with_callback(
            "datachannel",
            peer_datachannel_callback.as_ref().unchecked_ref(),
        )?;

        let channel = None;

        let closures = (
            None,
            None,
            None,
            None,
            peer_icecandidate_callback,
            peer_iceconnectionstatechange_callback,
            peer_connectionstatechange_callback,
            peer_icegatheringstatechange_callback,
            peer_datachannel_callback,
        );

        Ok(Self {
            waker,
            closures,
            peer,
            channel,
            their_peer_id,
            our_peer_id,
            role,
            offer_state,
            signalling_sender,
        })
    }

    async fn create_channel(&mut self) -> Result<(), FrontendError<Nothing>> {
        self.remove_channel_event_listners();
        if let Some(channel) = self.channel.take() {
            // TODO:
            channel.close();
        }

        let channel = {
            let mut config = RtcDataChannelInit::new();
            config.ordered(true);

            debug!("creating channel");
            let mut channel =
                self.peer.create_data_channel_with_data_channel_dict("client data", &config);
            channel.set_binary_type(RtcDataChannelType::Arraybuffer);

            channel
        };

        let waker = &self.waker;

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
                let message: Option<String> = event.data().as_string();
                debug!("channel_message callback: {message:?}");
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

        let channel_close_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |event: Event| {
                debug!("channel_close callback: {event:?}");
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        channel.add_event_listener_with_callback(
            "close",
            channel_close_callback.as_ref().unchecked_ref(),
        )?;

        self.channel = Some(channel);
        self.closures.0 = Some(channel_error_callback);
        self.closures.1 = Some(channel_message_callback);
        self.closures.2 = Some(channel_open_callback);
        self.closures.3 = Some(channel_close_callback);

        Ok(())
    }

    async fn send_offer_or_answer(
        &mut self,
        sender: &mut SocketSink<ClientMessage>,
    ) -> Result<(), FrontendError<Nothing>> {
        let (local_init, offer_state): (RtcSessionDescriptionInit, _) =
            if let Some(remote_description) = self.peer.remote_description() {
                debug!("sending answer to {}", self.their_peer_id);
                // If we have a remote description, we have already received an offer and should
                // answer it instead of sending our own offer
                (
                    JsFuture::from(self.peer.create_answer())
                        .await
                        .map_err(FrontendError::from)
                        .with_context(|| {
                            format!(
                                "create_answer. our_peer_id: {}, their_peer_id: {}",
                                self.our_peer_id, self.their_peer_id
                            )
                        })?
                        .into(),
                    OfferState::Answered,
                )
            } else {
                // Since we are offering, create the channel
                self.create_channel().await?;

                debug!("sending offer to {}", self.their_peer_id);
                (
                    JsFuture::from(self.peer.create_offer())
                        .await
                        .map_err(FrontendError::from)
                        .with_context(|| {
                            format!(
                                "create_offer. our_peer_id: {}, their_peer_id: {}",
                                self.our_peer_id, self.their_peer_id
                            )
                        })?
                        .into(),
                    OfferState::Offered,
                )
            };

        // Update our local description with the offer/answer
        JsFuture::from(self.peer.set_local_description(&local_init))
            .await
            .map_err(FrontendError::from)
            .with_context(|| {
                format!(
                    "set_local_description. our_peer_id: {}, their_peer_id: {}",
                    self.our_peer_id, self.their_peer_id
                )
            })?;

        let local_desc = self.peer.local_description().ok_or(FrontendError::Client {
            message: "Local description missing after it was set".to_string(),
        })?;

        let type_ = local_desc.type_();
        let sdp = local_desc.sdp();
        let offer_or_answer = Offer { type_: type_.into(), sdp };

        let peer = self.their_peer_id.clone();
        let message = match offer_state {
            OfferState::Offered => ClientRtc::Offer { offer: offer_or_answer, peer },
            _ => ClientRtc::Answer { answer: offer_or_answer, peer },
        };

        sender.send(message.into()).await?;
        self.offer_state = offer_state;

        Ok(())
    }

    async fn handle_answer(
        &mut self,
        sender: &mut SocketSink<ClientMessage>,
        answer: Offer,
    ) -> Result<(), FrontendError<Nothing>> {
        if self.peer.remote_description().is_none() {
            debug!("handling answer from: {}", self.their_peer_id);
            let remote_desc = {
                let mut desc = RtcSessionDescriptionInit::new(answer.type_.into());
                desc.sdp(&answer.sdp);
                desc
            };

            JsFuture::from(self.peer.set_remote_description(&remote_desc))
                .await
                .map_err(FrontendError::from)
                .with_context(|| {
                    format!(
                        "set_remote_description. our_peer_id: {}, their_peer_id: {}, sdp: {:?}",
                        self.our_peer_id, self.their_peer_id, answer.sdp
                    )
                })?;
        } else {
            warn!("handle_answer when remote_description.is_some");
        }

        Ok(())
    }

    async fn handle_offer(
        &mut self,
        sender: &mut SocketSink<ClientMessage>,
        offer: Offer,
    ) -> Result<(), FrontendError<Nothing>> {
        let accept_offer = match (self.peer.remote_description(), self.role) {
            (None, _) => {
                debug!("accepting offer from: {}", self.their_peer_id);
                true
            },
            (Some(_), PerfectRole::Impolite) => {
                warn!("Impolite peer dropping offer from: {}", self.their_peer_id);
                false
            },
            (Some(_), PerfectRole::Polite) => {
                warn!(
                    "Polite peer discarding our offer in favour of offer from: {}",
                    self.their_peer_id
                );
                true
            },
        };

        if accept_offer {
            let remote_desc = {
                let mut desc = RtcSessionDescriptionInit::new(offer.type_.into());
                desc.sdp(&offer.sdp);
                desc
            };

            JsFuture::from(self.peer.set_remote_description(&remote_desc))
                .await
                .map_err(FrontendError::from)
                .with_context(|| {
                    format!(
                        "set_remote_description. our_peer_id: {}, their_peer_id: {}, sdp: {:?}",
                        self.our_peer_id, self.their_peer_id, offer.sdp
                    )
                })?;

            // Send the answer
            self.send_offer_or_answer(sender).await?;
        }

        Ok(())
    }

    async fn handle_signalling(
        &mut self,
        sender: &mut SocketSink<ClientMessage>,
    ) -> Result<(), FrontendError<Nothing>> {
        self.send_offer_or_answer(sender).await?;
        Ok(())
    }

    async fn handle_ice_candidate(
        &mut self,
        sender: &mut SocketSink<ClientMessage>,
        candidate: IceCandidate,
    ) -> Result<(), FrontendError<Nothing>> {
        JsFuture::from(
            self.peer
                .add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(candidate.into()).as_ref()),
        )
        .await?;
        Ok(())
    }

    fn remove_channel_event_listners(&mut self) {
        if let Some(channel) = self.channel.as_ref() {
            for (event, closure) in [
                ("error", self.closures.0.as_ref().unwrap().as_ref()),
                ("message", self.closures.1.as_ref().unwrap().as_ref()),
                ("open", self.closures.2.as_ref().unwrap().as_ref()),
                ("open", self.closures.3.as_ref().unwrap().as_ref()),
            ] {
                let _ =
                    channel.remove_event_listener_with_callback(&event, closure.unchecked_ref());
            }

            self.closures.0 = None;
            self.closures.1 = None;
            self.closures.2 = None;
            self.closures.3 = None;
        }
    }

    fn close(mut self) {
        self.remove_channel_event_listners();

        for (event, closure) in [
            ("icecandidate", self.closures.4.as_ref()),
            ("iceconnectionstatechange", self.closures.5.as_ref()),
            ("connectionstatechange", self.closures.6.as_ref()),
            ("icegatheringstatechange", self.closures.7.as_ref()),
            ("datachannel", self.closures.4.as_ref()),
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
}

impl RtcInner {
    fn new(
        mut source: RtcSource,
        mut sender: SocketSink<ClientMessage>,
        waker: Rc<RefCell<Option<Waker>>>,
    ) -> Self {
        let peers: Arc<DashMap<PeerId, Peer>> = Default::default();
        let waker_ = Rc::clone(&waker);
        let peers_ = peers.clone();

        fn ensure_peer<'a>(
            waker: &Rc<RefCell<Option<Waker>>>,
            sender: &SocketSink<ClientMessage>,
            our_peer_id: &Option<PeerId>,
            their_peer_id: &PeerId,
            peers: &'a Arc<DashMap<PeerId, Peer>>,
        ) -> Result<(RefMut<'a, PeerId, Peer>, bool), FrontendError<Nothing>> {
            use dashmap::Entry::*;

            let our_peer_id = our_peer_id.as_ref().ok_or(FrontendError::Other {
                message: format!("Got peer signalling messages before our_peer_id was set"),
            })?;

            let peer_ref = match peers.entry(their_peer_id.clone()) {
                Occupied(v) => (v.into_ref(), false),
                Vacant(v) => {
                    let waker = Rc::clone(&waker);
                    let peer = Peer::new(
                        waker,
                        sender.clone(),
                        our_peer_id.clone(),
                        their_peer_id.clone(),
                    )?;
                    (v.insert(peer), true)
                },
            };

            Ok(peer_ref)
        }

        spawn_local(async move {
            let r = async move {
                let mut our_peer_id = None;

                loop {
                    let count = peers_.iter().count();
                    debug!(count, "peers");

                    if let Some(m) = source.receiver.next().await {
                        match m {
                            ServerRtc::PeerId(p) => {
                                debug!("got our_peer_id: {p:?}");
                                our_peer_id = Some(p);
                            },

                            ServerRtc::RoomPeers(peers) => {
                                debug!("RoomPeers: {peers:?}");

                                for their_peer_id in peers {
                                    let (mut peer, new) = ensure_peer(
                                        &waker_,
                                        &sender,
                                        &our_peer_id,
                                        &their_peer_id,
                                        &peers_,
                                    )?;
                                    debug!(new, "RoomPeers got peer: {their_peer_id}");
                                    peer.handle_signalling(&mut sender).await?;
                                }
                            },

                            ServerRtc::PeerOffer { offer, peer: their_peer_id } => {
                                let (mut peer, new) = ensure_peer(
                                    &waker_,
                                    &sender,
                                    &our_peer_id,
                                    &their_peer_id,
                                    &peers_,
                                )?;
                                debug!(new, "PeerOffer from: {their_peer_id}");
                                peer.handle_offer(&mut sender, offer).await?;
                            },

                            ServerRtc::PeerAnswer { answer, peer: their_peer_id } => {
                                let (mut peer, new) = ensure_peer(
                                    &waker_,
                                    &sender,
                                    &our_peer_id,
                                    &their_peer_id,
                                    &peers_,
                                )?;
                                debug!(new, "PeerAnswer from: {their_peer_id}");
                                peer.handle_answer(&mut sender, answer).await?;
                            },

                            ServerRtc::IceCandidate { candidate, peer: their_peer_id } => {
                                let (mut peer, new) = ensure_peer(
                                    &waker_,
                                    &sender,
                                    &our_peer_id,
                                    &their_peer_id,
                                    &peers_,
                                )?;
                                debug!(new, "IceCandidate from: {their_peer_id}");
                                peer.handle_ice_candidate(&mut sender, candidate).await?;
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

        Self { waker, peers }
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

        let inner = RtcInner::new(source, sender, waker);

        let inner = store_value(inner);

        Ok(Self { inner })
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
}
