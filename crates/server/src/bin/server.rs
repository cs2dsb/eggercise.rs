use std::{
    fs::remove_file,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};

use anyhow::Context;
use axum::{
    http::{HeaderName, HeaderValue, Method, StatusCode, Uri},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::Parser;
use deadpool_sqlite::{Config, Hook, Runtime};
use server::{
    cli::Cli, db, routes::auth::*, AppError, AppState
};
use shared::{
    api,
    configure_tracing, load_dotenv,
};
use tower_sessions_deadpool_sqlite_store::DeadpoolSqliteStore;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer};
use tracing::{debug, info, Level};
use webauthn_rs::{prelude::Url, WebauthnBuilder};

fn build_webauthn(args: &Cli) -> Result<webauthn_rs::Webauthn, anyhow::Error> {
    let rp_name = format!("eggercise.rs on {}", &args.webauthn_origin);
    let url = Url::parse(&args.webauthn_origin).with_context(|| {
        format!(
            "Parsing \"{}\" as webauthn origin URL",
            &args.webauthn_origin
        )
    })?;

    let builder = WebauthnBuilder::new(&args.webauthn_id, &url).with_context(|| {
        format!(
            "WebauthnBuilder::new({}, {})",
            &args.webauthn_id, &args.webauthn_origin
        )
    })?;

    Ok(builder.rp_name(&rp_name).build()?)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    load_dotenv()?;
    configure_tracing();

    let args = Cli::parse();
    debug!(?args);

    if args.debug_delete_database {
        let path = PathBuf::from_str(&args.sqlite_connection_string).unwrap();
        if path.exists() {
            remove_file(&path)?;
        }
    }

    // Run the migrations synchronously before creating the pool or launching the
    // server
    let ran = db::run_migrations(&args.sqlite_connection_string)?;
    info!("Ran {ran} db migrations");

    let webauthn = Arc::new(build_webauthn(&args)?);

    // Create a database pool to add into the app state
    let pool = Config::new(args.sqlite_connection_string.clone())
        .builder(Runtime::Tokio1)?
        .post_create(Hook::async_fn(|object, _| {
            Box::pin(async move {
                object
                    .interact(|conn| db::configure_new_connection(conn))
                    .await
                    .map_err(AppError::from)?
                    .map_err(AppError::from)?;
                Ok(())
            })
        }))
        .build()?;

    let session_store = DeadpoolSqliteStore::new(pool.clone());
    session_store.migrate().await?;

    let socket = SocketAddr::new(IpAddr::from_str(&args.bind_addr)?, args.port);

    let listener = TcpListener::bind(socket).await?;
    debug!("listening on {}", listener.local_addr()?);

    let state = AppState {
        pool,
        webauthn,
        args: Arc::new(args.clone()),
    };

    axum::serve(
        listener,
        Router::new()
            .route(api::Auth::RegisterStart.path(), post(register_start))
            .route(api::Auth::RegisterFinish.path(), post(register_finish))
            .route(api::Auth::LoginStart.path(), post(login_start))
            .route(api::Auth::LoginFinish.path(), post(login_finish))
            .route(
                api::Auth::RegisterNewKeyStart.path(),
                post(register_new_key_start),
            )
            .route(
                api::Auth::RegisterNewKeyFinish.path(),
                post(register_new_key_finish),
            )
            // The following routes require the user to be signed in
            .route(api::Object::User.path(), get(fetch_user))
            .route(api::Auth::CreateTemporaryLogin.path(), post(create_temporary_login))
            .route(api::Object::QrCode.id_path(), get(generate_qr_code))
            .route(api::Auth::TemporaryLogin.path(), get(temporary_login))
            .nest_service(
                "/wasm/service_worker.js",
                ServiceBuilder::new()
                    .layer(SetResponseHeaderLayer::if_not_present(
                        HeaderName::from_static("service-worker-allowed"),
                        HeaderValue::from_static("/"),
                    ))
                    .service(ServeFile::new(
                        args.assets_dir.join("wasm/service_worker.js"),
                    )),
            )
            .nest_service("/", ServeDir::new(&args.assets_dir))
            .layer(middleware::map_response(fallback_layer))
            .layer(
                ServiceBuilder::new()
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                            .on_response(DefaultOnResponse::new().level(Level::INFO)),
                    )
                    .layer(
                        SessionManagerLayer::new(session_store)
                            .with_secure(args.secure_sessions)
                            .with_expiry(Expiry::OnInactivity(Duration::days(
                                args.session_expiry_days,
                            ))),
                    ),
            )
            .with_state(state),
    )
    .await?;

    Ok(())
}

async fn fallback_layer(uri: Uri, method: Method, response: Response) -> impl IntoResponse {
    let code = response.status();

    match code {
        StatusCode::NOT_FOUND => Err(AppError::new(code, format!("Not found: {}", uri))),
        StatusCode::METHOD_NOT_ALLOWED => Err(AppError::new(
            code,
            format!("Method not allowed: {}: {}", method, uri),
        )),

        _ => Ok(response),
    }
}
