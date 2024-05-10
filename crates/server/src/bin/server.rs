use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

use axum::{
    extract::FromRef,
    http::{HeaderName, HeaderValue},
    routing::get,
    Json, Router,
};
use clap::Parser;
use deadpool_sqlite::{Config, Pool, Runtime};
use server::{AppError, DatabaseConnection};
use shared::*;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::{debug, Level};

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

    // Create a database pool to add into the app state
    let cfg = Config::new(args.sqlite_connection_string);
    let pool = cfg.create_pool(Runtime::Tokio1)?;

    let socket = SocketAddr::new(IpAddr::from_str(&args.bind_addr)?, args.port);

    let listener = TcpListener::bind(socket).await?;
    debug!("listening on {}", listener.local_addr()?);

    let state = AppState {
        pool,
    };

    axum::serve(
        listener,
        Router::new()
            .route("/db_test", get(db_test))
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

async fn db_test(
    DatabaseConnection(conn): DatabaseConnection,
) -> Result<Json<Vec<String>>, AppError> {
    tracing::info!("Connection: {:?}", conn);

    let results = conn
        .interact(|conn| {
            let mut stmt = conn.prepare_cached("SELECT name FROM DBSTAT")?;
            let r: Vec<String> = stmt
                .query_map((), |r| Ok(r.get::<_, String>(0)?))?
                .collect::<Result<_, _>>()?;
            Ok::<_, anyhow::Error>(r)
        })
        .await??;

    Ok(Json(results))
}
