use std::{net::{IpAddr, SocketAddr}, path::PathBuf, str::FromStr};

use axum::{http::{HeaderName, HeaderValue}, Router};
use shared::*;
use clap::Parser;
use tokio::net::TcpListener;
use tracing::{debug, Level};
use tower_http::{
    services::{ServeDir, ServeFile}, set_header::SetResponseHeaderLayer, trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer}
};
use tower::ServiceBuilder;

#[derive(Debug, Parser)]
#[clap(name = "eggercise server")]
struct Cli {
    #[clap(long, env, default_value = "static")]
    static_path: PathBuf,
    #[clap(long, env, default_value = "server.sqlite")]
    sqlite_connection_string: String,
    #[clap(long, env, default_value = "64")]
    database_command_channel_bound: usize,
    #[clap(long, env, default_value = "8080")]
    port: u16,
    #[clap(long, env, default_value = "127.0.0.1")]
    bind_addr: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    load_dotenv()?;
    configure_tracing();

    let args = Cli::parse();
    debug!(?args);

    let socket = SocketAddr::new(
        IpAddr::from_str(&args.bind_addr)?, 
        args.port);

    let listener = TcpListener::bind(socket).await?;
    debug!("listening on {}", listener.local_addr()?);

    axum::serve(
        listener, 
            Router::new()
                // Add the header to allow service worker in non-root path to set a root scope
                .nest_service("/wasm/service_worker.js", ServiceBuilder::new()
                    .layer(SetResponseHeaderLayer::if_not_present(
                        HeaderName::from_static("service-worker-allowed"), 
                        HeaderValue::from_static("/")))
                    .service(ServeFile::new(args.static_path.join("wasm/service_worker.js"))))
                .nest_service("/", ServeDir::new(&args.static_path))
                .layer(TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO))))
        .await
        .unwrap();

    Ok(())
}
