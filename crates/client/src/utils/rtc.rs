use std::{any::type_name, cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc, task::Waker};

use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    select, SinkExt, StreamExt,
};
use gloo::timers::future::IntervalStream;
use leptos::{provide_context, spawn_local, store_value, use_context, StoredValue};
use reconnecting_websocket::SocketSink;
use shared::{
    api::error::{FrontendError, Nothing, ResultContext},
    types::{
        rtc::PeerId,
        websocket::{ClientMessage, ClientRtc, IceCandidate, RoomPeer, Sdp, SdpType, ServerRtc},
    },
};
use tracing::{debug, error, info, warn};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::{ArrayBuffer, JsString, Reflect, Uint8Array},
    ErrorEvent, Event, MessageEvent, RtcConfiguration, RtcDataChannel, RtcDataChannelEvent,
    RtcDataChannelInit, RtcDataChannelState, RtcDataChannelType, RtcIceConnectionState,
    RtcPeerConnection, RtcPeerConnectionIceEvent, RtcPeerConnectionState, RtcSdpType,
    RtcSessionDescriptionInit, RtcSignalingState,
};

/// The role this peer is taking
/// See <https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API/Perfect_negotiation>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PerfectRole {
    Polite,
    Impolite,
}

struct Closures {
    channel_error: Option<Closure<dyn FnMut(ErrorEvent)>>,
    channel_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    channel_open: Option<Closure<dyn FnMut(RtcDataChannelEvent)>>,
    channel_close: Option<Closure<dyn FnMut(Event)>>,
    peer_icecandidate: Option<Closure<dyn FnMut(RtcPeerConnectionIceEvent)>>,
    peer_iceconnectionstatechange: Option<Closure<dyn FnMut(Event)>>,
    peer_connectionstatechange: Option<Closure<dyn FnMut(Event)>>,
    peer_icegatheringstatechange: Option<Closure<dyn FnMut(Event)>>,
    peer_datachannel: Option<Closure<dyn FnMut(RtcDataChannelEvent)>>,
    peer_negotiationneeded: Option<Closure<dyn FnMut(Event)>>,
    peer_signalingstatechange: Option<Closure<dyn FnMut(Event)>>,
}
struct Peer {
    waker: Rc<RefCell<Option<Waker>>>,

    #[allow(unused)]
    signaling_sender: SocketSink<ClientMessage>,
    channel: Option<RtcDataChannel>,
    peer: RtcPeerConnection,
    our_peer_id: PeerId,
    their_peer_id: PeerId,
    role: PerfectRole,
    queued_candidates: Vec<IceCandidate>,
    peer_update_sender: UnboundedSender<(PeerId, PeerUpdate)>,
    our_petname: String,
    their_petname: String,

    // Hang on to them for de-registering
    closures: Closures,
}

macro_rules! remove_handler {
    ($target:expr, $type_:expr, $old_listener:expr $(,)?) => {{
        if let Some(old_listener) = $old_listener.take() {
            $target
                .remove_event_listener_with_callback($type_, old_listener.as_ref().unchecked_ref())
                .map_err(FrontendError::from)
                .with_context(|| {
                    format!("removing old event listener for \"{}\" from {:?}", $type_, $target)
                })?;
        }
    }};
}

macro_rules! replace_handler {
    ($target:expr, $type_:expr, $listener:expr, $old_listener:expr $(,)?) => {{
        // Grab the old one first but don't remove it
        let old_listener = $old_listener.take();

        $target
            .add_event_listener_with_callback($type_, $listener.as_ref().unchecked_ref())
            .map_err(FrontendError::from)
            .with_context(|| {
                format!("adding event listener for \"{}\" to {:?}", $type_, $target)
            })?;

        *$old_listener = Some($listener);

        // Remove the old one
        if let Some(old_listener) = old_listener {
            $target
                .remove_event_listener_with_callback($type_, old_listener.as_ref().unchecked_ref())
                .map_err(FrontendError::from)
                .with_context(|| {
                    format!("removing old event listener for \"{}\" from {:?}", $type_, $target)
                })?;

            // Forget the closure so JS GC can clean it up AFTER any pending calls
            // As weak references are supported by all major browsers this doesn't cause a memory
            // leak
            old_listener.forget();
        }
    }};
}

#[derive(Debug)]
enum PeerUpdate {
    DataChannel(RtcDataChannel),
    DataChannelClosed,
    Destroy,
}

impl From<RtcDataChannel> for PeerUpdate {
    fn from(value: RtcDataChannel) -> Self {
        Self::DataChannel(value)
    }
}

impl Peer {
    fn new(
        waker: Rc<RefCell<Option<Waker>>>,
        signaling_sender: SocketSink<ClientMessage>,
        our_peer_id: PeerId,
        their_peer_id: PeerId,
        peer_update_sender: UnboundedSender<(PeerId, PeerUpdate)>,
        our_petname: &str,
        their_petname: String,
    ) -> Result<Self, FrontendError<Nothing>> {
        let compound_petname = format!("{our_petname} [{their_petname}]");

        // Since both parties know the peer IDs this will result in them picking the correct roles
        let role =
            if our_peer_id > their_peer_id { PerfectRole::Polite } else { PerfectRole::Impolite };

        let peer = {
            let config = RtcConfiguration::new();

            // TODO: peer identity is actually for the remote peer and requires validation
            // config.peer_identity(Some(&our_peer_id.to_string()));

            RtcPeerConnection::new_with_configuration(&config)?
        };

        let signaling_sender_ = signaling_sender.clone();
        let their_peer_id_ = their_peer_id.clone();
        let petname_ = compound_petname.clone();
        let peer_icecandidate_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
                if let Some(candidate) = event.candidate() {
                    debug!("{petname_}: peer_icecandidate callback: {}", candidate.candidate());
                    let mut signaling_sender_ = signaling_sender_.clone();
                    let their_peer_id_ = their_peer_id_.clone();
                    let petname_ = petname_.clone();
                    // TODO: might be preferable to have a channel with a sync send to send this
                    // back to the main loop?
                    spawn_local(async move {
                        if let Err(e) = signaling_sender_
                            .send(
                                ClientRtc::IceCandidate {
                                    candidate: candidate.into(),
                                    peer_id: their_peer_id_.clone(),
                                }
                                .into(),
                            )
                            .await
                        {
                            error!(
                                "{petname_}: Error sending ice candidate to signaling server: \
                                 {e:?}"
                            );
                        }
                    });
                } else {
                    debug!("{petname_}: peer_icecandidate callback: Null candidiate");
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

        let petname_ = compound_petname.clone();
        let peer_iceconnectionstatechange_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            let target_key = JsValue::from_str("target");
            Closure::wrap(Box::new(move |event: Event| {
                let peer: RtcPeerConnection =
                    Reflect::get(&event, &target_key).expect("event.target").into();
                let state = peer.ice_connection_state();
                debug!("{petname_}: peer_iceconnectionstatechange callback: {state:?}");
                match state {
                    // TODO: do an ice restart
                    RtcIceConnectionState::Failed => {},
                    // cancel ice restart? Some browser differences apparently
                    RtcIceConnectionState::Disconnected => {},
                    _ => {},
                }
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        peer.add_event_listener_with_callback(
            "iceconnectionstatechange",
            peer_iceconnectionstatechange_callback.as_ref().unchecked_ref(),
        )?;

        let peer_update_sender_ = peer_update_sender.clone();
        let their_peer_id_ = their_peer_id.clone();
        let petname_ = compound_petname.clone();
        let peer_connectionstatechange_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            let target_key = JsValue::from_str("target");
            Closure::wrap(Box::new(move |event: Event| {
                let peer: RtcPeerConnection =
                    Reflect::get(&event, &target_key).expect("event.target").into();
                let state = peer.connection_state();
                debug!("{petname_}: peer_connectionstatechange callback: {:?}", state);
                match state {
                    RtcPeerConnectionState::Closed
                    // TODO: maybe wait if we try an ICE restart?
                    | RtcPeerConnectionState::Failed
                    | RtcPeerConnectionState::Disconnected => {
                        peer_update_sender_
                            .unbounded_send((their_peer_id_.clone(), PeerUpdate::Destroy))
                            .expect("peer_connectionstatechange_callback unbounded_send");
                    },
                    _ => {},
                }
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        peer.add_event_listener_with_callback(
            "connectionstatechange",
            peer_connectionstatechange_callback.as_ref().unchecked_ref(),
        )?;

        let petname_ = compound_petname.clone();
        let peer_icegatheringstatechange_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            let target_key = JsValue::from_str("target");
            Closure::wrap(Box::new(move |event: Event| {
                let peer: RtcPeerConnection =
                    Reflect::get(&event, &target_key).expect("event.target").into();

                debug!(
                    "{petname_}: peer_icegatheringstatechange callback: {:?}",
                    peer.ice_gathering_state()
                );
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        peer.add_event_listener_with_callback(
            "icegatheringstatechange",
            peer_icegatheringstatechange_callback.as_ref().unchecked_ref(),
        )?;

        let peer_update_sender_ = peer_update_sender.clone();
        let their_peer_id_ = their_peer_id.clone();
        let petname_ = compound_petname.clone();
        let peer_datachannel_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |event: RtcDataChannelEvent| {
                let channel = event.channel();
                debug!("{petname_}: peer_datachannel callback: {:?}", channel);

                peer_update_sender_
                    .unbounded_send((their_peer_id_.clone(), channel.into()))
                    .expect("peer_datachannel_callback unbounded_send");

                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        peer.add_event_listener_with_callback(
            "datachannel",
            peer_datachannel_callback.as_ref().unchecked_ref(),
        )?;

        let petname_ = compound_petname.clone();
        let peer_negotiationneeded_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |event: Event| {
                debug!("{petname_}: peer_negotiationneeded callback: {:?}", event);

                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        peer.add_event_listener_with_callback(
            "negotiationneeded",
            peer_negotiationneeded_callback.as_ref().unchecked_ref(),
        )?;

        let petname_ = compound_petname.clone();
        let peer_signalingstatechange_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            let target_key = JsValue::from_str("target");
            Closure::wrap(Box::new(move |event: Event| {
                let peer: RtcPeerConnection =
                    Reflect::get(&event, &target_key).expect("event.target").into();

                debug!(
                    "{petname_}: peer_signalingstatechange callback: {:?}",
                    peer.signaling_state()
                );

                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        peer.add_event_listener_with_callback(
            "signalingstatechange",
            peer_signalingstatechange_callback.as_ref().unchecked_ref(),
        )?;

        let channel = None;
        let queued_candidates = Vec::new();
        let our_petname = our_petname.to_string();

        let closures = Closures {
            channel_error: None,
            channel_message: None,
            channel_open: None,
            channel_close: None,
            peer_icecandidate: Some(peer_icecandidate_callback),
            peer_iceconnectionstatechange: Some(peer_iceconnectionstatechange_callback),
            peer_connectionstatechange: Some(peer_connectionstatechange_callback),
            peer_icegatheringstatechange: Some(peer_icegatheringstatechange_callback),
            peer_datachannel: Some(peer_datachannel_callback),
            peer_negotiationneeded: Some(peer_negotiationneeded_callback),
            peer_signalingstatechange: Some(peer_signalingstatechange_callback),
        };

        Ok(Self {
            waker,
            closures,
            peer,
            channel,
            their_peer_id,
            our_peer_id,
            role,
            signaling_sender,
            queued_candidates,
            peer_update_sender,
            our_petname,
            their_petname,
        })
    }

    fn compound_petname(&self) -> String {
        format!("{} [{}]", self.our_petname, self.their_petname)
    }

    async fn handle_datachannel(
        &mut self,
        channel: RtcDataChannel,
    ) -> Result<(), FrontendError<Nothing>> {
        self.create_channel(Some(channel)).await?;
        Ok(())
    }

    fn close_datachannel(&mut self) -> Result<(), FrontendError<Nothing>> {
        if let Some(channel) = self.channel.take() {
            channel.close();
            remove_handler!(&channel, "error", &mut self.closures.channel_error,);
            remove_handler!(&channel, "message", &mut self.closures.channel_message,);
            remove_handler!(&channel, "open", &mut self.closures.channel_open,);
            remove_handler!(&channel, "close", &mut self.closures.channel_close,);
        }

        Ok(())
    }

    async fn create_channel(
        &mut self,
        channel: Option<RtcDataChannel>,
    ) -> Result<(), FrontendError<Nothing>> {
        let compound_petname = self.compound_petname();

        if let Some(channel) = self.channel.take() {
            debug!("{compound_petname}: Closing data channel");
            channel.close();
        }

        let channel = channel.unwrap_or_else(|| {
            let mut config = RtcDataChannelInit::new();
            config.ordered(true);

            debug!("{compound_petname}: creating channel");
            self.peer.create_data_channel_with_data_channel_dict("client data", &config)
        });

        channel.set_binary_type(RtcDataChannelType::Arraybuffer);

        let waker = &self.waker;

        let petname_ = compound_petname.clone();
        let channel_error_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            // TODO: Not sure this is the right event type
            Closure::wrap(Box::new(move |event: ErrorEvent| {
                debug!("{petname_}: channel_error callback: {event:?}");
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        replace_handler!(
            &channel,
            "error",
            channel_error_callback,
            &mut self.closures.channel_error
        );

        let petname_ = compound_petname.clone();
        let channel_message_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |event: MessageEvent| {
                let data = event.data();
                if data.has_type::<JsString>() {
                    debug!("{petname_}: channel_message callback string: {:?}", data.as_string());
                } else if data.has_type::<ArrayBuffer>() {
                    let u8_array = Uint8Array::new(&data);
                    let bytes = u8_array.to_vec();
                    let string = String::from_utf8_lossy(&bytes);
                    debug!("{petname_}: channel_message callback Arraybuffer: {:?}", string);
                } else {
                    debug!(
                        "{petname_}: channel_message callback unknown type: {:?}",
                        data.js_typeof().as_string()
                    );
                }
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        replace_handler!(
            &channel,
            "message",
            channel_message_callback,
            &mut self.closures.channel_message
        );

        let petname_ = compound_petname.clone();
        let channel_open_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |event: RtcDataChannelEvent| {
                debug!("{petname_}: channel_open callback: {event:?}");
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        replace_handler!(&channel, "open", channel_open_callback, &mut self.closures.channel_open);

        let petname_ = compound_petname.clone();
        let peer_update_sender = self.peer_update_sender.clone();
        let their_peer_id = self.their_peer_id.clone();
        let channel_close_callback: Closure<dyn FnMut(_)> = {
            let waker = Rc::clone(&waker);
            Closure::wrap(Box::new(move |_event: Event| {
                debug!("{petname_}: channel_close callback");

                peer_update_sender
                    .unbounded_send((their_peer_id.clone(), PeerUpdate::DataChannelClosed))
                    .expect("channel_close_callback unbounded_send");

                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(_)>)
        };
        replace_handler!(
            &channel,
            "close",
            channel_close_callback,
            &mut self.closures.channel_close
        );

        self.channel = Some(channel);

        Ok(())
    }

    async fn perform_signaling(
        &mut self,
        sender: &mut SocketSink<ClientMessage>,
        offer: Option<Sdp>,
    ) -> Result<(), FrontendError<Nothing>> {
        use PerfectRole::*;
        use RtcSignalingState::*;
        use SdpType::{Answer as AnswerT, Offer as OfferT};

        let compound_petname = self.compound_petname();

        // Could calculate just_established from the others but this is more explicit
        let (set_remote, response, just_established) = match (
            offer.as_ref().map(|o| o.type_),
            self.peer.remote_description().is_some(),
            self.peer.local_description().is_some(),
            self.peer.signaling_state(),
            self.role,
        ) {
            // Unexpected offer or answer on an established connection
            (Some(_), true, true, Stable, _) => {
                warn!(
                    "{compound_petname}: got offer when remote desc is set and signaling_state is \
                     stable. Ignoring. (their_peer_id: {})",
                    self.their_peer_id
                );

                // Don't set or send anything
                (false, None, false)
            },

            // Unexpected call without offer or answer
            (None, true, true, Stable, _) => {
                debug!(
                    "{compound_petname}: perform_signaling called with no offer on established \
                     connection. Ignoring. (their_peer_id: {})",
                    self.their_peer_id
                );

                // Don't set or send anything
                (false, None, false)
            },

            // Conflict/glare offer on unestablished when we are polite
            (Some(OfferT), false, true, HaveLocalOffer, Polite) => {
                warn!(
                    "{compound_petname}: glare (polite): got offer when we have local desc. \
                     Rolling back. (their_peer_id: {})",
                    self.their_peer_id
                );

                // We will have created a channel as part of our offer which will be replaced when
                // accepting their offer Close channel
                if let Some(channel) = self.channel.take() {
                    debug!("{}: Closing data channel", self.our_petname);
                    channel.close();
                }

                // Rollback local desc
                let rollback = RtcSessionDescriptionInit::new(RtcSdpType::Rollback);
                JsFuture::from(self.peer.set_local_description(&rollback))
                    .await
                    .map_err(FrontendError::from)
                    .with_context(|| {
                        format!(
                            "{compound_petname}: set_local_description(rollback). (their_peer_id: \
                             {})",
                            self.their_peer_id
                        )
                    })?;

                // Set remote and send back an answer
                (true, Some(AnswerT), true)
            },

            // Conflict/glare offer on unestablished when we are impolite
            (Some(OfferT), false, true, HaveLocalOffer, Impolite) => {
                warn!(
                    "{compound_petname}: glare (impolite): got offer when we have local desc. \
                     Ignoring. (their_peer_id: {})",
                    self.their_peer_id
                );

                // Don't set or send anything
                (false, None, false)
            },

            // Answer to our offer
            (Some(AnswerT), false, true, HaveLocalOffer, _) => {
                debug!(
                    "{compound_petname}: got answer to our offer. Accepting. (their_peer_id: {})",
                    self.their_peer_id
                );

                // Set remote but send nothing
                (true, None, true)
            },

            // Offer to open on unestablished
            (Some(OfferT), false, false, Stable, _) => {
                debug!(
                    "{compound_petname}: got offer, remote desc == none, local desc == none, \
                     Stable. Answering. (their_peer_id: {})",
                    self.their_peer_id
                );

                // Set remote and send back an answer
                (true, Some(AnswerT), true)
            },

            // We're initiating
            (None, false, false, Stable, _) => {
                debug!(
                    "{compound_petname}: initiating offer, remote desc == none, local desc == \
                     none, Stable. Sending offer. (their_peer_id: {})",
                    self.their_peer_id
                );

                // Since we are offering, create the channel
                self.create_channel(None).await?;

                // Don't set remote, send offer
                (false, Some(OfferT), false)
            },

            // Should be impossible
            (o, r, l, s, p) => {
                error!(
                    "{compound_petname}: unexpected situatuion - offer: {o:?}, remote desc: \
                     {r:?}, local desc: {l:?}, state: {s:?}, {p:?}. (their_peer_id: {})",
                    self.their_peer_id
                );
                unreachable!();
            },
        };

        if set_remote {
            let offer = offer.expect(&format!(
                "{compound_petname}: set_remote but offer was None. (their_peer_id: {})",
                self.their_peer_id
            ));

            let remote_desc = {
                let mut desc = RtcSessionDescriptionInit::new(offer.type_.into());
                desc.sdp(&offer.sdp);
                desc
            };

            // Set remote description
            JsFuture::from(self.peer.set_remote_description(&remote_desc))
                .await
                .map_err(FrontendError::from)
                .with_context(|| {
                    format!(
                        "{compound_petname}: set_remote_description. our_peer_id: {}, \
                         their_peer_id: {}, sdp: {:?}",
                        self.our_peer_id, self.their_peer_id, offer.sdp
                    )
                })?;
        }

        let local_desc: Option<RtcSessionDescriptionInit> = match response {
            Some(AnswerT) => {
                debug!("{}: creating answer", self.our_petname);
                // Create answer
                let desc = JsFuture::from(self.peer.create_answer())
                    .await
                    .map_err(FrontendError::from)
                    .with_context(|| {
                        format!(
                            "{compound_petname}: create_answer. our_peer_id: {}, their_peer_id: {}",
                            self.our_peer_id, self.their_peer_id
                        )
                    })?
                    .into();

                Some(desc)
            },
            Some(OfferT) => {
                debug!("creating offer");
                // Create offer
                let desc = JsFuture::from(self.peer.create_offer())
                    .await
                    .map_err(FrontendError::from)
                    .with_context(|| {
                        format!(
                            "{compound_petname}: create_offer. our_peer_id: {}, their_peer_id: {}",
                            self.our_peer_id, self.their_peer_id
                        )
                    })?
                    .into();

                Some(desc)
            },
            _ => None,
        };

        if let Some(local_desc) = local_desc {
            debug!("{}: setting local desc", self.our_petname);
            // Set local description
            JsFuture::from(self.peer.set_local_description(&local_desc))
                .await
                .map_err(FrontendError::from)
                .with_context(|| {
                    format!(
                        "{compound_petname}: set_local_description(answer). our_peer_id: {}, \
                         their_peer_id: {}",
                        self.our_peer_id, self.their_peer_id
                    )
                })?;

            let local_desc = self.peer.local_description().ok_or(FrontendError::Client {
                message: format!(
                    "{}: Local description missing after it was set",
                    self.our_petname
                ),
            })?;

            let type_ = local_desc.type_().into();
            let sdp = Sdp { type_, sdp: local_desc.sdp() };
            let peer_id = self.their_peer_id.clone();
            let petname = self.our_petname.clone();
            let message = ClientRtc::Sdp { sdp, peer_id, petname };

            debug!("{compound_petname}: sending signaling {:?}", type_);

            sender.send(message.into()).await?;
        }

        if just_established {
            let state = self.peer.signaling_state();
            let ris = self.peer.remote_description().is_some();
            assert!(
                state == RtcSignalingState::Stable && ris,
                "{compound_petname}: If just_established == true, State should be Stable \
                 (actually {state:?}) and remote desc must be some (actually {ris}",
            );
        }

        if just_established && self.queued_candidates.len() > 0 {
            let candidates = self.queued_candidates.drain(..).collect::<Vec<_>>();
            for candidate in candidates {
                self.handle_ice_candidate(sender, candidate).await?;
            }
        }

        Ok(())
    }

    async fn handle_ice_candidate(
        &mut self,
        _sender: &mut SocketSink<ClientMessage>,
        candidate: IceCandidate,
    ) -> Result<(), FrontendError<Nothing>> {
        if self.peer.signaling_state() != RtcSignalingState::Stable
            || self.peer.remote_description().is_none()
        {
            warn!(
                "{}: got ice candidate before connected and stable. queuing",
                self.compound_petname()
            );
            self.queued_candidates.push(candidate);
            return Ok(());
        }

        JsFuture::from(
            self.peer
                .add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(candidate.into()).as_ref()),
        )
        .await?;

        Ok(())
    }

    fn close(mut self) -> Result<(), FrontendError<Nothing>> {
        self.close_datachannel()?;

        remove_handler!(&self.peer, "icecandidate", &mut self.closures.peer_icecandidate,);
        remove_handler!(
            &self.peer,
            "iceconnectionstatechange",
            &mut self.closures.peer_iceconnectionstatechange
        );
        remove_handler!(
            &self.peer,
            "connectionstatechange",
            &mut self.closures.peer_connectionstatechange
        );
        remove_handler!(
            &self.peer,
            "icegatheringstatechange",
            &mut self.closures.peer_icegatheringstatechange
        );
        remove_handler!(&self.peer, "datachannel", &mut self.closures.peer_datachannel,);
        remove_handler!(&self.peer, "negotiationneeded", &mut self.closures.peer_negotiationneeded,);
        remove_handler!(
            &self.peer,
            "signalingstatechange",
            &mut self.closures.peer_signalingstatechange
        );

        Ok(())
    }

    async fn send<T: AsRef<[u8]> + Debug>(
        &mut self,
        data: T,
    ) -> Result<(), FrontendError<Nothing>> {
        if let Some(channel) = self.channel.as_ref() {
            let compound_petname = self.compound_petname();
            let ready_state = channel.ready_state();
            if ready_state == RtcDataChannelState::Open {
                debug!("{compound_petname}: Sending: {data:?}");
                channel.send_with_u8_array(data.as_ref())?;
            } else {
                warn!("{compound_petname}: Attempted to send on {ready_state:?} channel");
            }
        }
        Ok(())
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
    _waker: Rc<RefCell<Option<Waker>>>,
}

impl RtcInner {
    fn new(
        mut source: RtcSource,
        mut signaling_sender: SocketSink<ClientMessage>,
        waker: Rc<RefCell<Option<Waker>>>,
    ) -> Self {
        let waker_ = Rc::clone(&waker);

        fn get_peer<'a>(
            their_peer_id: &PeerId,
            peers: &'a mut HashMap<PeerId, Peer>,
        ) -> Option<&'a mut Peer> {
            peers.get_mut(their_peer_id)
        }

        fn ensure_peer<'a>(
            waker: &Rc<RefCell<Option<Waker>>>,
            sender: &SocketSink<ClientMessage>,
            our_peer_id: &PeerId,
            their_peer_id: &PeerId,
            peers: &'a mut HashMap<PeerId, Peer>,
            peer_sender: &UnboundedSender<(PeerId, PeerUpdate)>,
            our_petname: &str,
            their_petname: String,
        ) -> Result<(&'a mut Peer, bool), FrontendError<Nothing>> {
            let new = if !peers.contains_key(their_peer_id) {
                let waker = Rc::clone(&waker);
                let peer = Peer::new(
                    waker,
                    sender.clone(),
                    our_peer_id.clone(),
                    their_peer_id.clone(),
                    peer_sender.clone(),
                    our_petname,
                    their_petname,
                )?;
                peers.insert(their_peer_id.clone(), peer);
                true
            } else {
                false
            };

            Ok((peers.get_mut(their_peer_id).unwrap(), new))
        }

        spawn_local(async move {
            let mut peers: HashMap<PeerId, Peer> = Default::default();
            let (channel_sender, channel_receiver) = mpsc::unbounded();

            let r = async move {
                let our_peer_id = PeerId::new();
                let mut petname = "<unnamed>".to_string();

                // Announce our peer id to the server, this also kicks off joining our user room
                signaling_sender.send(ClientRtc::Announce { peer_id: our_peer_id.clone() }.into()).await?;

                let mut channel_receiver = channel_receiver.fuse();
                let mut keepalive_interval = IntervalStream::new(5000).fuse();

                let mut boop = 0_usize;

                loop {
                    let count = peers.iter().count();
                    debug!(count, "{petname}: peers");

                    select! {
                        _ = keepalive_interval.next() => {
                            for peer in peers.values_mut() {
                                peer.send(format!("{boop}")).await?;
                                boop += 1;
                            }
                        },

                        r = channel_receiver.next() => match r {
                            None => {
                                // We currently never close the sender
                                unreachable!();
                            },
                            Some((their_peer_id, PeerUpdate::Destroy)) => {
                                if let Some(peer) = peers.remove(&their_peer_id) {
                                    debug!("{}: Got destroy for peer: {their_peer_id}", peer.compound_petname());
                                    peer.close()?;
                                } else {
                                    unreachable!("{petname}: Can't get PeerUpdate from non-existent peer");
                                }
                            },
                            Some((their_peer_id, update)) => {
                                if let Some(peer) = get_peer(&their_peer_id, &mut peers) {
                                    match update {
                                        PeerUpdate::DataChannel(channel) => {
                                            debug!("{}: Got datachannel for peer: {their_peer_id}", peer.compound_petname());
                                            peer.handle_datachannel(channel).await?;
                                        },

                                        PeerUpdate::DataChannelClosed => {
                                            debug!("{}: Got datachannel close for peer: {their_peer_id}", peer.compound_petname());
                                            peer.close_datachannel()?;
                                        },

                                        // Handled above
                                        PeerUpdate::Destroy => unreachable!(),
                                    }
                                } else {
                                    error!("{petname}: got PeerUpdate for a peer that doesn't exist!");
                                }
                            },
                        },

                        r = source.receiver.next() => match r {
                            Some(m) => {
                                match m {
                                    ServerRtc::Petname(name) => {
                                        debug!("Assigned petname: {name}");
                                        petname = name;
                                    },
                                    ServerRtc::RoomPeers(room_peers) => {
                                        debug!("{petname}: RoomPeers: {room_peers:?}");

                                        for RoomPeer { peer_id: their_peer_id, petname: their_petname } in room_peers {
                                            let (peer, new) = ensure_peer(
                                                &waker_,
                                                &signaling_sender,
                                                &our_peer_id,
                                                &their_peer_id,
                                                &mut peers,
                                                &channel_sender,
                                                &petname,
                                                their_petname,
                                            )?;
                                            debug!(new, "{}: RoomPeers got peer: {their_peer_id}", peer.compound_petname());
                                            peer.perform_signaling(&mut signaling_sender, None).await?;
                                        }
                                    },

                                    ServerRtc::PeerSdp { sdp, peer_id: their_peer_id, petname: their_petname } => {
                                        let (peer, new) = ensure_peer(
                                            &waker_,
                                            &signaling_sender,
                                            &our_peer_id,
                                            &their_peer_id,
                                            &mut peers,
                                            &channel_sender,
                                            &petname,
                                            their_petname,
                                        )?;
                                        debug!(new, "{}: PeerSdp (type: {:?}) from: {their_peer_id}", peer.compound_petname(), sdp.type_);
                                        peer.perform_signaling(&mut signaling_sender, Some(sdp)).await?;
                                    },

                                    ServerRtc::IceCandidate { candidate, peer_id: their_peer_id } => {
                                        if let Some(peer) = get_peer(&their_peer_id,&mut peers) {
                                            debug!("{}: IceCandidate from: {their_peer_id}", peer.compound_petname());
                                            peer.handle_ice_candidate(&mut signaling_sender, candidate).await?;
                                        } else {
                                            error!("{petname}: Got ice candidate from non-existent peer ({their_peer_id})");
                                        }
                                    },
                                }
                            },
                            None => {
                                info!("{petname}: source closed");
                                break;
                            },
                        }
                    }
                }
                Ok::<_, FrontendError<Nothing>>(())
            }
            .await;

            if let Err(e) = r {
                error!("Error from rtc inner task: {e:?}");
            }
        });

        Self { _waker: waker }
    }
}

#[derive(Clone)]
pub struct Rtc {
    _inner: StoredValue<RtcInner>,
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

        Ok(Self { _inner: inner })
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
