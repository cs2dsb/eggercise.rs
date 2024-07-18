use std::sync::Arc;

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use dashmap::DashMap;
use loole::Sender;
use shared::types::{
    rtc::PeerId,
    websocket::{IceCandidate, Offer},
};

use crate::{AppState, PeerIdState, UserState};

#[derive(Debug)]
pub enum ClientControlMessage {
    Login(UserState),
    Logout,
    RtcOffer { offer: Offer, peer: PeerId },
    RtcAnswer { answer: Offer, peer: PeerId },
    RtcIceCandidate { candidate: IceCandidate, peer: PeerId },
}

type ClientKey = PeerId;

#[derive(Debug, Clone)]
pub struct Client {
    key: ClientKey,
    sender: Sender<ClientControlMessage>,
}

impl Client {
    pub fn key(&self) -> &ClientKey {
        &self.key
    }

    pub fn new(key: ClientKey, sender: Sender<ClientControlMessage>) -> Self {
        Self { key, sender }
    }

    pub async fn send(
        &self,
        msg: ClientControlMessage,
    ) -> Result<(), loole::SendError<ClientControlMessage>> {
        self.sender.send_async(msg).await
    }
}

type ClientMap = DashMap<ClientKey, Client>;

#[derive(Debug, Clone, Default)]
pub struct Clients(Arc<ClientMap>);

impl Clients {
    pub fn add(&self, client: Client) -> Option<Client> {
        self.0.insert(client.key().clone(), client)
    }

    pub fn get(&self, key: &ClientKey) -> Option<Client> {
        self.0.get(key).map(|v| v.value().clone())
    }

    pub fn remove(&self, key: &ClientKey) -> Option<Client> {
        self.0.remove(key).map(|v| v.1)
    }

    pub fn call_with<F, E>(&self, key: &ClientKey, f: F) -> Result<(), E>
    where
        F: Fn(Option<&Client>) -> Result<(), E>,
    {
        let ref_ = self.0.get(key);
        let v = ref_.as_ref().map(|v| v.value());
        f(v)
    }
}

impl From<Arc<ClientMap>> for Clients {
    fn from(args: Arc<ClientMap>) -> Self {
        Clients(args)
    }
}

impl FromRef<AppState> for Clients {
    fn from_ref(state: &AppState) -> Self {
        state.websocket_clients.clone()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Clients
where
    S: Send + Sync,
    Clients: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Clients::from_ref(state))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Client
where
    S: Send + Sync,
    Clients: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let peer_id_state = PeerIdState::from_request_parts(parts, state).await?;
        let clients = Clients::from_ref(state);

        let client = clients.get(&*peer_id_state).ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to find client for peer_id: {peer_id_state:?}"),
            )
        })?;

        Ok(client)
    }
}
