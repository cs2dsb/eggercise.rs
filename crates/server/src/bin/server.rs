use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

use axum::{
    extract::{FromRef, Path},
    http::{HeaderName, HeaderValue},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use deadpool_sqlite::{Config, Hook, Pool, Runtime};
use server::{db::{self, model::{NewUser, User}, DatabaseConnection}, AppError };
use shared::*;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::{debug, info, instrument, Level};

#[derive(Debug, Parser)]
#[clap(name = "eggercise server")]
struct Cli {
    #[clap(long, env, default_value = "assets")]
    assets_dir: PathBuf,
    #[clap(long, env, default_value = "egg.sqlite")]
    sqlite_connection_string: String,
    #[clap(long, env, default_value = "64")]
    database_command_channel_bound: usize,
    #[clap(long, env, default_value = "8080")]
    port: u16,
    #[clap(long, env, default_value = "127.0.0.1")]
    bind_addr: String,
}

#[derive(Debug, Clone)]
struct AppState {
    pool: Pool,
}

impl FromRef<AppState> for Pool {
    fn from_ref(state: &AppState) -> Self {
        // pool uses an Arc internally so clone is cheap
        state.pool.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    load_dotenv()?;
    configure_tracing();

    let args = Cli::parse();
    debug!(?args);

    // Run the migrations synchronously before creating the pool or launching the server
    let ran = db::run_migrations(&args.sqlite_connection_string)?;
    info!("Ran {ran} db migrations");

    // Create a database pool to add into the app state
    let pool = Config::new(args.sqlite_connection_string)
        .builder(Runtime::Tokio1)?
        .post_create(Hook::async_fn(|object, _| {
            Box::pin(async move {
                object.interact(|conn| {
                    db::configure_new_connection(conn)
                })
                .await
                .map_err(AppError::from)?
                .map_err(AppError::from)?;
                Ok(())
            })
        }))
        .build()?;

    let socket = SocketAddr::new(IpAddr::from_str(&args.bind_addr)?, args.port);

    let listener = TcpListener::bind(socket).await?;
    debug!("listening on {}", listener.local_addr()?);

    let state = AppState {
        pool,
    };

    axum::serve(
        listener,
        Router::new()
        .route("/api/user/:id", get(fetch_user))
        .route("/api/user", post(create_user))
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
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            )
            .with_state(state),
    )
    .await?;

    Ok(())
}

#[instrument]
async fn create_user(
    DatabaseConnection(conn): DatabaseConnection,
    Json(new_user): Json<NewUser>,
) -> Result<Json<User>, AppError> {
    let results = conn.interact(|conn|
        Ok::<_, anyhow::Error>(User::create(conn, new_user)?))
        .await??;

    Ok(Json(results))
}

#[instrument]
async fn fetch_user(
    DatabaseConnection(conn): DatabaseConnection,
    Path(id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let results = conn.interact(move |conn|
        Ok::<_, anyhow::Error>(User::fetch(conn, id)?))
        .await??;

    Ok(Json(results))
}