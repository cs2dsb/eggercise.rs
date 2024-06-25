mod webauthn;
use std::{io::Cursor, sync::Arc};

use axum::extract::FromRef;
use deadpool_sqlite::Pool;
pub use webauthn::*;

mod args;
pub use args::*;

use crate::cli::Cli;

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

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: Pool,
    pub webauthn: Arc<webauthn_rs::Webauthn>,
    pub args: Arc<Cli>,
    pub vapid_pub_key: VapidPubKey,
    pub vapid_private_key: VapidPrivateKey,
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

impl FromRef<AppState> for Pool {
    fn from_ref(state: &AppState) -> Self {
        // pool uses an Arc internally so clone is cheap
        state.pool.clone()
    }
}

impl FromRef<AppState> for Arc<webauthn_rs::Webauthn> {
    fn from_ref(state: &AppState) -> Self {
        state.webauthn.clone()
    }
}

impl FromRef<AppState> for Arc<Cli> {
    fn from_ref(state: &AppState) -> Self {
        state.args.clone()
    }
}
