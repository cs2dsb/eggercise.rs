use std::{
    fs::remove_file, net::{IpAddr, SocketAddr}, path::PathBuf, str::FromStr, sync::Arc
};

use anyhow::Context;
use axum::{
    extract::{FromRef, Path}, http::{HeaderName, HeaderValue, Method, StatusCode, Uri}, middleware, response::{IntoResponse, Response}, routing::{get, post}, Json, Router
};
use clap::Parser;
use deadpool_sqlite::{Config, Hook, Pool, Runtime};
use server::{db::{self, DatabaseConnection}, AppError, PasskeyRegistrationState, SessionValue, Webauthn };
use shared::{api, configure_tracing, load_dotenv, model::{Credential, NewUser, NewUserWithPasskey, RegistrationUser, User}};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing::{debug, info, Level};
use webauthn_rs::{prelude::{CreationChallengeResponse, RegisterPublicKeyCredential, Url, Uuid}, WebauthnBuilder};

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
    #[clap(long, env, default_value = "false")]
    secure_sessions: bool,
    #[arg(long, env, default_value = "http://127.0.0.1:8080")]
    webauthn_origin: String,
    #[arg(long, env, default_value = "127.0.0.1")]
    webauthn_id: String,

    /// Deletes the database before starting the main program for debug purposes
    #[arg(long, env, default_value = "false")]
    debug_delete_database: bool,
}

#[derive(Debug, Clone)]
struct AppState {
    pool: Pool,
    webauthn: Arc<webauthn_rs::Webauthn>,
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

fn build_webauthn(args: &Cli) -> Result<webauthn_rs::Webauthn, anyhow::Error> {
    let rp_name =format!("eggercise.rs on {}", &args.webauthn_origin);
    let url = Url::parse(&args.webauthn_origin)
        .with_context(|| format!("Parsing \"{}\" as webauthn origin URL", &args.webauthn_origin))?;

    let builder = WebauthnBuilder::new(&args.webauthn_id, &url)
        .with_context(|| format!("WebauthnBuilder::new({}, {})", &args.webauthn_id, &args.webauthn_origin))?;

    Ok(builder
        .rp_name(&rp_name)
        .build()?)
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

    // Run the migrations synchronously before creating the pool or launching the server
    let ran = db::run_migrations(&args.sqlite_connection_string)?;
    info!("Ran {ran} db migrations");

    let webauthn = Arc::new(build_webauthn(&args)?);

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
        webauthn,
    };

    axum::serve(
        listener,
        Router::new()
            // .route(api::Auth::RegisterStart.path(), post(register_start))
            .route(api::Auth::RegisterFinish.path(),  post(register_finish))
            .route(api::Auth::Login.path(),  post(login))
            .route(api::Object::User.id_path(), get(fetch_user))
            .route(api::Object::User.path(), post(create_user))
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
            .layer(ServiceBuilder::new()
                .layer(TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)))
                .layer(SessionManagerLayer::new(MemoryStore::default())
                    .with_secure(args.secure_sessions))
            )
            .with_state(state)
    )
    .await?;

    Ok(())
}


// #[axum::debug_handler]
// Based on https://github.com/kanidm/webauthn-rs/blob/628599aa47b5c120e7f29cce8c526af532fba9ce/tutorial/server/axum/src/auth.rs#L52
#[allow(dead_code)]
async fn register_start(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    Json(reg_user): Json<RegistrationUser>,
) -> Result<Json<CreationChallengeResponse>, AppError> {
    // Remove the existing challenge
    session.take_passkey_registration_state().await?;

    // TODO: validate username
    let (user_id, existing_key_ids) = {
        let username = reg_user.username.clone(); 
        conn.interact(move |conn| {
            // First get the uuid associated with the given username, if any
            let user_id = User::fetch_by_username(conn, username)?
                .map(|u| u.id);

            // Then fetch the existing passkeys if the user exists
            Ok::<_, anyhow::Error>(match user_id {
                None => (Uuid::new_v4().into(), Vec::new()),
                Some(user_id) => {
                    let passkeys = Credential::fetch_passkeys(conn, &user_id)?
                        .into_iter()
                        // We only want the ID
                        .map(|p| p.cred_id().to_owned())
                        .collect::<Vec<_>>();
                    (
                        user_id, 
                        passkeys,
                    )
                },
            })
        }).await??
    };

    // Start the registration 
    let (creation_challenge_response, passkey_registration) = webauthn.start_passkey_registration(
        *user_id,
        &reg_user.username,
        // TODO: display name
        &reg_user.username,
        Some(existing_key_ids),
    )?;

    // Stash the registration
    session.set_passkey_registration_state(
        PasskeyRegistrationState::new(reg_user.username, *user_id, passkey_registration)).await?;

    // Send the challenge back to the client
    Ok(Json(creation_challenge_response))
}

async fn register_finish(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    Json(register_public_key_credential): Json<RegisterPublicKeyCredential>,
) -> Result<StatusCode, AppError> {
    // Get the challenge from the session
    let PasskeyRegistrationState {username, id, passkey_registration } = session
        .take_passkey_registration_state()
        .await?
        .ok_or(anyhow::anyhow!("Current session doesn't contain a PasskeyRegistrationState. Client error or replay attack?"))?;

    // Attempt to complete the passkey registration with the provided public key
    let passkey = webauthn.finish_passkey_registration(&register_public_key_credential, &passkey_registration)?;
    
    // Create the new user with their passkey
    let new_user = NewUserWithPasskey::new(id, username, passkey);
    conn.interact(move |conn| 
        Ok::<_, anyhow::Error>(new_user.create(conn)?))
        .await??;

    Ok(StatusCode::OK)
}

#[allow(unused_variables)]
async fn login(
    DatabaseConnection(_conn): DatabaseConnection,
    // Json(): Json<>,
) -> Result<Json<()>, AppError> {
    // let results = conn.interact(|conn|
    //     Ok::<_, anyhow::Error>(User::create(conn, new_user)?))
    //     .await??;

    // Ok(Json(results))
    todo!()
}


#[allow(unused_variables)]
async fn create_user(
    DatabaseConnection(conn): DatabaseConnection,
    Json(new_user): Json<NewUser>,
) -> Result<Json<User>, AppError> {
    let results = conn.interact(|conn|
        Ok::<_, anyhow::Error>(User::create(conn, new_user)?))
        .await??;

    Ok(Json(results))
}


#[allow(unused_variables)]
async fn fetch_user(
    DatabaseConnection(_conn): DatabaseConnection,
    Path(_id): Path<i64>,
) -> Result<Json<User>, AppError> {
    todo!()
    // let results = conn.interact(move |conn|
    //     Ok::<_, anyhow::Error>(User::fetch_by_id(conn, id)?))
    //     .await??;

    // Ok(Json(results))
}


async fn fallback_layer(
    uri: Uri,
    method: Method,
    response: Response,
) -> impl IntoResponse {
    let code = response.status();

    match code {
        StatusCode::NOT_FOUND => 
            Err(AppError::new(code, format!("Not found: {}", uri))),
        StatusCode::METHOD_NOT_ALLOWED =>
            Err(AppError::new(code, format!("Method not allowed: {}: {}", method, uri))),
        
        _ => Ok(response)
    }
}