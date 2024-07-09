use std::{net::SocketAddr, sync::Arc};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use dashmap::DashMap;
use loole::Sender;
use shared::model::UserId;

use crate::AppState;

#[derive(Debug)]
pub enum ClientControlMessage {}

type ClientKey = (UserId, SocketAddr);

#[derive(Debug)]
pub struct Client {
    key: ClientKey,
    sender: Sender<ClientControlMessage>,
}

impl Client {
    pub fn key(&self) -> &ClientKey {
        &self.key
    }

    pub fn new(
        user_id: UserId,
        socket_address: SocketAddr,
        sender: Sender<ClientControlMessage>,
    ) -> Self {
        Self { key: (user_id, socket_address), sender }
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
        Ok(Clients::from_ref(state).into())
    }
}
