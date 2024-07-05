use std::collections::HashMap;

use axum::{extract::FromRef, routing::post, Json, Router};
use deadpool_sqlite::Pool;
use serde::Deserialize;
use shared::api::error::{Nothing, ServerError};
use tower_http::limit::RequestBodyLimitLayer;
use tower_sessions::Session;
use tracing::{debug, error, info, span, trace, warn, Level};

use crate::{db::DatabaseConnection, UserState};

const LOG_MAX_BYTES: usize = 1024;

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Pool: FromRef<S>,
{
    Router::new()
        .route("/", post(handler))
        .layer(RequestBodyLimitLayer::new(LOG_MAX_BYTES))
}

#[derive(Debug, Deserialize)]
struct Payload {
    #[serde(default)]
    level: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(flatten)]
    other: HashMap<String, serde_json::Value>,
}

impl Payload {
    fn level(&self) -> Level {
        let level = self
            .level
            .as_ref()
            .map_or(String::new(), |v| v.to_lowercase());
        match level.as_ref() {
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::TRACE,
        }
    }
    fn message<'a>(&'a self) -> &'a str {
        self.message.as_ref().map_or("<No message>", |v| v.as_str())
    }
}

async fn handler(
    DatabaseConnection(_conn): DatabaseConnection,
    user_state: Option<UserState>,
    session: Session,
    Json(payload): Json<Payload>,
) -> Result<Json<()>, ServerError<Nothing>> {
    let level = payload.level();

    let client_log_span = match &level {
        &Level::DEBUG => span!(parent: None, Level::DEBUG, "client_log"),
        &Level::TRACE => span!(parent: None, Level::TRACE, "client_log"),
        &Level::INFO => span!(parent: None, Level::INFO, "client_log"),
        &Level::WARN => span!(parent: None, Level::WARN, "client_log"),
        &Level::ERROR => span!(parent: None, Level::ERROR, "client_log"),
    };
    let _guard = client_log_span.enter();

    let user_id = user_state.map(|v| v.id.to_string());
    let session_id = session.id().map(|v| v.to_string());

    match level {
        Level::TRACE => {
            trace!(target: "log", parent: &client_log_span, session_id = session_id, user_id = user_id, fields = ?payload.other, "\"{}\"", payload.message())
        }
        Level::DEBUG => {
            debug!(target: "log", parent: &client_log_span, session_id = session_id, user_id = user_id, fields = ?payload.other, "\"{}\"", payload.message())
        }
        Level::INFO => {
            info!(target: "log", parent: &client_log_span, session_id = session_id, user_id = user_id, fields = ?payload.other, "\"{}\"", payload.message())
        }
        Level::WARN => {
            warn!(target: "log", parent: &client_log_span, session_id = session_id, user_id = user_id, fields = ?payload.other, "\"{}\"", payload.message())
        }
        Level::ERROR => {
            error!(target: "log", parent: &client_log_span, session_id = session_id, user_id = user_id, fields = ?payload.other, "\"{}\"", payload.message())
        }
    }

    Ok(Json(()))
}
