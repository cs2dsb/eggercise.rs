mod webauthn;
use std::sync::Arc;

use axum::extract::FromRef;
use deadpool_sqlite::Pool;
pub use webauthn::*;

mod args;
pub use args::*;

use crate::cli::Cli;

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: Pool,
    pub webauthn: Arc<webauthn_rs::Webauthn>,
    pub args: Arc<Cli>,
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
