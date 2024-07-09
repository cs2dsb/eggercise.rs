#![allow(unused)]
use std::{any::type_name, sync::Arc};

use leptos::{provide_context, use_context};
use shared::{
    api::error::{FrontendError, Nothing},
    rtc::{
        peer_connector::Connector as _,
        signalling_client::{self, Client as _},
        Builder as _, PeerConnector, PeerMap, SignallingClient,
    },
    types::rtc::{PeerId, RoomId},
};

use crate::utils::location::{host, protocol};

#[derive(Clone)]
pub struct Rtc {
    connector: Arc<PeerConnector<RoomId, PeerId>>,
    peers: Arc<PeerMap>,
    signalling_client: Arc<SignallingClient<RoomId, PeerId>>,
}

impl Rtc {
    async fn new() -> Result<Self, FrontendError<Nothing>> {
        assert!(
            use_context::<Self>().is_none(),
            "Call to Rtc::new() when there's already one in the context"
        );

        let rtc_connector =
            PeerConnector::with_base(format!("{}://{}", protocol()?, host()?)).build()?;
        let signalling_client = rtc_connector.build_signalling_client()?;

        Ok(Self {
            connector: Arc::new(rtc_connector),
            peers: Default::default(),
            signalling_client: Arc::new(signalling_client),
        })
    }

    pub async fn provide_context() -> Result<(), FrontendError<Nothing>> {
        if use_context::<Self>().is_none() {
            let v = Self::new().await?;
            provide_context(v);
        }
        Ok(())
    }

    pub fn use_rtc() -> Self {
        use_context::<Self>().expect(&format!("{} missing from context", type_name::<Self>()))
    }
}

// #![allow(unused)]
// use std::any::type_name;
//
// use gloo::utils::errors::JsError;
// use leptos::{provide_context, use_context};
// use shared::api::{
// error::{FrontendError, Nothing},
// Object,
// };
// use tracing::{ info, debug };
// use wasm_bindgen::{convert::FromWasmAbi, prelude::Closure, JsCast, JsValue};
// use wasm_bindgen_futures::JsFuture;
// use web_sys::{js_sys::Function, ErrorEvent, Event, MessageEvent,
// RtcConfiguration, RtcDataChannel, RtcDataChannelEvent, RtcDataChannelInit,
// RtcDataChannelType, RtcIceCandidateInit, RtcOfferOptions, RtcPeerConnection,
// RtcPeerConnectionIceEvent, RtcSessionDescription, RtcSessionDescriptionInit};
//
// use crate::{api::send_offer, utils::location::{host, protocol}};
//
//
// #[derive(Clone)]
// pub struct Rtc {
// peer: RtcPeerConnection
// }
//
// impl Rtc {
// async fn new() -> Result<Self, FrontendError<Nothing>> {
// assert!(
// use_context::<Self>().is_none(),
// "Call to Rtc::new() when there's already one in the context"
// );
//
//
// let peer = {
// let mut config = RtcConfiguration::new();
// config.peer_identity(Some("client_name"));
//
// RtcPeerConnection::new_with_configuration(&config)?
// };
//
// let channel  = {
// let mut config = RtcDataChannelInit::new();
// config.ordered(true);
//
// let mut channel = peer.create_data_channel_with_data_channel_dict("client
// data", &config); channel.set_binary_type(RtcDataChannelType::Arraybuffer);
//
// wrap_callback(
// |cb| channel.set_onerror(Some(cb)),
// Self::on_error,
// );
//
// wrap_callback(
// |cb| channel.set_onmessage(Some(cb)),
// Self::on_message,
// );
//
// wrap_callback(
// |cb| channel.set_onopen(Some(cb)),
// Self::on_open,
// );
//
// channel
// };
//
// wrap_callback(
// |cb| peer.set_onconnectionstatechange(Some(cb)),
// Self::on_connectionstatechange,
// );
//
// wrap_callback(
// |cb| peer.set_onicegatheringstatechange(Some(cb)),
// Self::on_icegatheringstatechange,
// );
//
// wrap_callback(
// |cb| peer.set_onicecandidate(Some(cb)),
// Self::on_icecandidate,
// );
//
// This is requred or Firefox disconnects when it fires
// wrap_callback(
// |cb| peer.set_oniceconnectionstatechange(Some(cb)),
// Self::on_iceconnectionstatechange,
// );
//
// let offer: RtcSessionDescriptionInit = {
// let config = RtcOfferOptions::new();
// let offer: RtcSessionDescription =
// JsFuture::from(peer.create_offer_with_rtc_offer_options(&config)).await?.
// into();
//
// TODO: is this right? The callback returns a SessionDescription but the
// set_local_description in web_sys only takes the init version 🤷
// JsValue::from(offer).into()
// };
//
// JsFuture::from(peer.set_local_description(&offer)).await?;
//
// let sdp = peer.local_description()
// .ok_or(FrontendError::Client { message: "Local description missing after it
// was set".to_string() })? .sdp();
//
// let response = send_offer(sdp).await
// .map_err(|e| FrontendError::Client { message: format!("RTC offer response
// error: {e:?}") })?;
//
// info!("Got rtc offer response: {:?}", response);
// let answer = {
// let mut answer = RtcSessionDescriptionInit::new(response.type_.into());
// answer.sdp(&response.sdp);
// answer
// };
// JsFuture::from(peer.set_remote_description(&answer)).await?;
// info!("Remote description set successfully");
//
// let candidate : RtcIceCandidateInit = { todo!() };
// JsFuture::from(peer.add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(&
// candidate))).await?; info!("Ice candidate added successfully");
//
// Ok(Self { peer })
// }
//
// TODO: Not sure this is the right event type
// fn on_error(event: ErrorEvent) {
// debug!("Rtc error: {:?}", event.message());
// }
//
// fn on_message(event: MessageEvent) {
// debug!("Rtc message: {:?}", event.data());
// }
//
// fn on_open(event: RtcDataChannelEvent) {
// debug!("Rtc open: {:?}", event);
// }
//
// fn on_icecandidate(event: RtcPeerConnectionIceEvent) {
// debug!("Rtc icecandidate: {:?}", event);
// }
//
// fn on_iceconnectionstatechange(event: Event) {
// debug!("Rtc iceconnectionstatechange: {:?}", event);
// }
//
// fn on_connectionstatechange(event: Event) {
// debug!("Rtc connectionstatechange: {:?}", event);
// }
//
// fn on_icegatheringstatechange(event: Event) {
// debug!("Rtc icegatheringstatechange: {:?}", event);
// }
//
// pub async fn provide_context() -> Result<(), FrontendError<Nothing>> {
// if use_context::<Self>().is_none() {
// let ws = Self::new().await?;
// provide_context(ws);
// }
// Ok(())
// }
//
// pub fn use_rtc() -> Self {
// use_context::<Self>().expect(&format!("{} missing from context",
// type_name::<Self>())) }
// }
//
