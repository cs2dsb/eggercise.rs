use std::io::Cursor;

use axum::extract::FromRef;

use crate::AppState;

#[derive(Debug, Clone)]
pub struct VapidPubKey(Vec<u8>);

impl VapidPubKey {
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn cursor(&self) -> Cursor<&[u8]> {
        Cursor::new(self.bytes())
    }
}

impl From<Vec<u8>> for VapidPubKey {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl Into<Vec<u8>> for VapidPubKey {
    fn into(self) -> Vec<u8> {
        self.0
    }
}
#[derive(Debug, Clone)]
pub struct VapidPrivateKey(Vec<u8>);

impl VapidPrivateKey {
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn cursor(&self) -> Cursor<&[u8]> {
        Cursor::new(self.bytes())
    }
}

impl From<Vec<u8>> for VapidPrivateKey {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl Into<Vec<u8>> for VapidPrivateKey {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl FromRef<AppState> for VapidPubKey {
    fn from_ref(state: &AppState) -> Self {
        state.vapid_pub_key.clone()
    }
}

impl FromRef<AppState> for VapidPrivateKey {
    fn from_ref(state: &AppState) -> Self {
        state.vapid_private_key.clone()
    }
}
