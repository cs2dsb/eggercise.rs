use std::sync::Arc;

use axum::extract::FromRef;

use crate::{cli::Cli, state::AppState};

impl FromRef<AppState> for Arc<Cli> {
    fn from_ref(state: &AppState) -> Self {
        state.args.clone()
    }
}
