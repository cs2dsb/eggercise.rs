use std::{net::SocketAddr, sync::Arc};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use dashmap::{DashMap, Entry};
use loole::Sender;
use shared::types::{
    rtc::PeerId,
    websocket::{IceCandidate, Sdp},
};

use crate::{AppState, SessionId, UserState};

#[derive(Debug)]
pub enum ClientControlMessage {
    Login(UserState),
    Logout,
    RtcStp { sdp: Sdp, peer_id: PeerId, petname: String },
    RtcIceCandidate { candidate: IceCandidate, peer_id: PeerId },
}

type ClientKey = PeerId;
type ClientBySessionIdKey = SessionId;

#[derive(Debug, Clone)]
pub struct Client {
    socket_addr: SocketAddr,
    sender: Sender<ClientControlMessage>,
}

impl Client {
    pub fn new(socket_addr: SocketAddr, sender: Sender<ClientControlMessage>) -> Self {
        Self { socket_addr, sender }
    }

    pub async fn send(
        &self,
        msg: ClientControlMessage,
    ) -> Result<(), loole::SendError<ClientControlMessage>> {
        self.sender.send_async(msg).await
    }
}

#[derive(Debug, Clone)]
pub struct SessionClients {
    pub clients: Vec<Client>,
}

type ClientMap = DashMap<ClientKey, Client>;
type ClientBySessionIdMap = DashMap<ClientBySessionIdKey, SessionClients>;

#[derive(Debug, Clone, Default)]
pub struct Clients(Arc<ClientMap>);

#[derive(Debug, Clone, Default)]
pub struct ClientsBySessionId(Arc<ClientBySessionIdMap>);

impl Clients {
    pub fn add(&self, key: ClientKey, client: Client) -> Option<Client> {
        self.0.insert(key, client)
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

impl ClientsBySessionId {
    pub fn add(&self, key: ClientBySessionIdKey, client: Client) {
        match self.0.entry(key.clone()) {
            Entry::Occupied(mut v) => {
                v.get_mut().clients.push(client);
            },
            Entry::Vacant(v) => {
                v.insert(SessionClients { clients: vec![client] });
            },
        }
    }

    pub fn get(&self, key: &ClientBySessionIdKey) -> Option<SessionClients> {
        self.0.get(key).map(|v| v.value().clone())
    }

    pub fn remove(&self, key: &ClientBySessionIdKey, socket_addr: &SocketAddr) {
        let remove_all = if let Some(mut v) = self.0.get_mut(key) {
            v.clients.retain(|c| &c.socket_addr != socket_addr);
            v.clients.is_empty()
        } else {
            false
        };

        if remove_all {
            self.0.remove(key);
        }
    }
}

impl From<Arc<ClientMap>> for Clients {
    fn from(args: Arc<ClientMap>) -> Self {
        Clients(args)
    }
}

impl From<Arc<ClientBySessionIdMap>> for ClientsBySessionId {
    fn from(args: Arc<ClientBySessionIdMap>) -> Self {
        ClientsBySessionId(args)
    }
}

impl FromRef<AppState> for Clients {
    fn from_ref(state: &AppState) -> Self {
        state.websocket_clients.clone()
    }
}

impl FromRef<AppState> for ClientsBySessionId {
    fn from_ref(state: &AppState) -> Self {
        state.websocket_clients_by_user_id.clone()
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
impl<S> FromRequestParts<S> for ClientsBySessionId
where
    S: Send + Sync,
    ClientsBySessionId: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(ClientsBySessionId::from_ref(state))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for SessionClients
where
    S: Send + Sync,
    ClientsBySessionId: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session_id = SessionId::from_request_parts(parts, state).await?;
        let clients = ClientsBySessionId::from_ref(state);

        let client = clients.get(&session_id).ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to find client for session_id: {session_id:?}"),
            )
        })?;

        Ok(client)
    }
}
