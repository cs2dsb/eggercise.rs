use serde::{Deserialize, Serialize};

use super::ServerMessage;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerUser {
    /// User logged in
    Login,
    /// User logged out
    Logout,
}

impl From<ServerUser> for ServerMessage {
    fn from(value: ServerUser) -> Self {
        Self::User(value)
    }
}

#[cfg(feature = "backend")]
impl TryFrom<ServerUser> for axum::extract::ws::Message {
    type Error = <ServerMessage as TryInto<axum::extract::ws::Message>>::Error;

    fn try_from(message: ServerUser) -> Result<Self, Self::Error> {
        let message: ServerMessage = message.into();
        message.try_into()
    }
}
