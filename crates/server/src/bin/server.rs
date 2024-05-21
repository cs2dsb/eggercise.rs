#![allow(unused)]
use std::{
    fs::remove_file, net::{IpAddr, SocketAddr}, path::PathBuf, str::FromStr, sync::Arc
};

use anyhow::{ensure, Context};
use axum::{
    extract::{FromRef, Path}, http::{HeaderName, HeaderValue, Method, StatusCode, Uri}, middleware, response::{IntoResponse, Response}, routing::{get, post}, Json, Router
};
use chrono::Utc;
use clap::Parser;
use deadpool_sqlite::{Config, Hook, Pool, Runtime};
use server::{db::{self, DatabaseConnection}, AppError, PasskeyAuthenticationState, PasskeyRegistrationState, SessionValue, UserState, Webauthn };
use shared::{api::{self, error::{Nothing, ServerError}, response_errors::{FetchError, LoginError, RegisterError}}, configure_tracing, ensure_server, load_dotenv, model::{Credential, LoginUser, NewUser, NewUserWithPasskey, RegistrationUser, User, UserId}, other_error, unauthorized_error};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, SessionManagerLayer};
use tracing::{debug, info, Level};
use webauthn_rs::{prelude::{CreationChallengeResponse, PasskeyAuthentication, PublicKeyCredential, RegisterPublicKeyCredential, RequestChallengeResponse, Url, Uuid}, WebauthnBuilder};

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
    #[arg(long, env, default_value = "30")]
    session_expiry_days: i64,


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
            .route(api::Auth::RegisterStart.path(), post(register_start))
            .route(api::Auth::RegisterFinish.path(), post(register_finish))
            .route(api::Auth::LoginStart.path(), post(login_start))
            .route(api::Auth::LoginFinish.path(), post(login_finish))
            .route(api::Auth::RegisterNewKeyStart.path(), post(register_new_key_start))
            .route(api::Auth::RegisterNewKeyFinish.path(), post(register_new_key_finish))
            .route(api::Object::User.path(), get(fetch_user))
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
                    .with_secure(args.secure_sessions)
                    .with_expiry(Expiry::OnInactivity(Duration::days(args.session_expiry_days))))
            )
            .with_state(state)
    )
    .await?;

    Ok(())
}


// #[axum::debug_handler]
// Based on https://github.com/kanidm/webauthn-rs/blob/628599aa47b5c120e7f29cce8c526af532fba9ce/tutorial/server/axum/src/auth.rs#L52
async fn register_start(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    Json(reg_user): Json<RegistrationUser>,
) -> Result<Json<CreationChallengeResponse>, ServerError<RegisterError>> {
    // Remove the existing challenge
    // session.take_passkey_registration_state().await?;
    session.take_passkey_registration_state().await?;
    
    if reg_user.username.len() < 4 {
        Err(RegisterError::UsernameInvalid { message: "Username needs to be at least 4 characters".to_string() })?;
    }

    let (existing, user_id) = {
        let username = reg_user.username.clone(); 
        conn.interact(move |conn| {
            // Get the uuid associated with the given username, if any
            let user_id = User::fetch_by_username(conn, username)?
                .map(|u| u.id);

            Ok::<_, ServerError<_>>(match user_id {
                None => (false, Uuid::new_v4().into()),
                Some(uuid) => (true, uuid),
            })
        }).await??
    };

    if existing {
        Err(RegisterError::UsernameUnavailable)?;
    }

    // Start the registration 
    let (creation_challenge_response, passkey_registration) = webauthn.start_passkey_registration(
        *user_id,
        &reg_user.username,
        // TODO: display name
        &reg_user.username,
        None,
    )?;

    // Stash the registration
    session.set_passkey_registration_state(
        PasskeyRegistrationState::new(reg_user.username, user_id, passkey_registration)).await?;

    // Send the challenge back to the client
    Ok(Json(creation_challenge_response.into()))
}

async fn register_finish(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    Json(register_public_key_credential): Json<RegisterPublicKeyCredential>,
) -> Result<Json<()>, ServerError<Nothing>> {
    // Get the challenge from the session
    let PasskeyRegistrationState {username, id, passkey_registration } = session
        .take_passkey_registration_state()
        .await?
        .ok_or(unauthorized_error!("Current session doesn't contain a PasskeyRegistrationState. Client error or replay attack?"))?;

    // Attempt to complete the passkey registration with the provided public key
    let passkey = webauthn.finish_passkey_registration(&register_public_key_credential, &passkey_registration)?;
    
    // Create the new user with their passkey
    let new_user = NewUserWithPasskey::new(id, username, passkey);
    conn.interact(move |conn| 
        Ok::<_, ServerError<_>>(new_user.create(conn)?))
        .await??;

    Ok(Json(()))
}


async fn register_new_key_start(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    user_state: UserState,
) -> Result<Json<CreationChallengeResponse>, ServerError<RegisterError>> {
    // Remove the existing challenge
    // session.take_passkey_registration_state().await?;
    session.take_passkey_registration_state().await?;
    
    let user_id = user_state.id;

    let (user, existing_key_ids) = {
        conn.interact(move |conn| {
            // We need the username for the challenge so fetch the full user
            let user = user_id.fetch_full_user(conn)?;
            // Fetch the existing passkeys for this user
            let passkeys = Credential::fetch_passkeys(conn, &*user_id)?
                .into_iter()
                // We only want the ID for this step
                .map(|p| p.cred_id().to_owned())
                .collect::<Vec<_>>();

            Ok::<_, ServerError<_>>((user, passkeys))
        }).await??
    };

    if existing_key_ids.is_empty() {
        // Log the user out
        let _ = session.take_user_state().await?;
        Err(unauthorized_error!("No existing keys found. It is now impossible to log in to this account"))?;
    }

    // Start the registration challenge
    let (creation_challenge_response, passkey_registration) = webauthn.start_passkey_registration(
        *user.id,
        &user.username,
        // TODO: display name
        &user.username,
        Some(existing_key_ids),
    )?;

    // Stash the registration
    session.set_passkey_registration_state(
        PasskeyRegistrationState::new(user.username, user.id, passkey_registration)).await?;

    // Send the challenge back to the client
    Ok(Json(creation_challenge_response.into()))
}
    
async fn register_new_key_finish(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    user_state: UserState,
    Json(register_public_key_credential): Json<RegisterPublicKeyCredential>,
) -> Result<Json<()>, ServerError<Nothing>> {
    // Get the challenge from the session
    let PasskeyRegistrationState {username, id, passkey_registration } = session
        .take_passkey_registration_state()
        .await?
        .ok_or(unauthorized_error!("Current session doesn't contain a PasskeyRegistrationState. Client error or replay attack?"))?;

    // Attempt to complete the passkey registration with the provided public key
    let passkey = webauthn.finish_passkey_registration(&register_public_key_credential, &passkey_registration)?;
    
    let result = {
        conn.interact(move |conn| {
            // Get the user first
            let user = user_state.id.fetch_full_user(conn)
                .map_err(|e| (true, e.into()))?;
            
            // Add the new passkey
            user.add_passkey(conn, passkey)
                .map_err(|e| (false, e))?;

            Ok::<_, (bool, ServerError<_>)>(())
        })
        .await?
    };

    if let Err((logout, err)) = result {
        // Log the user out because there was no User in the database for the given id
        let _ = session.take_user_state().await?;
        Err(err)?;
    }
    
    Ok(Json(()))
}

async fn login_start(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    Json(login_user): Json<LoginUser>,
) -> Result<Json<RequestChallengeResponse>, ServerError<LoginError>> {
    // Remove the existing challenge
    session.take_passkey_authentication_state().await?;
    
    if login_user.username.len() < 4 {
        Err(LoginError::UsernameInvalid { message: "Username needs to be at least 4 characters".to_string() })?;
    }

    let (user, existing_passkeys) = {
        let username = login_user.username.clone(); 
        conn.interact(move |conn| {
            // First get the user associated with the given username, if any
            let user = User::fetch_by_username(conn, username)?;

            // Then fetch the existing passkeys if the user exists
            Ok::<_, ServerError<_>>(match user {
                None => (None, None),
                Some(user) => {
                    let passkeys = Credential::fetch_passkeys(conn, &user.id)?
                        .into_iter()
                        .collect::<Vec<_>>();
                    (
                        Some(user), 
                        Some(passkeys),
                    )
                },
            })
        }).await??
    };

    if user.is_none() {
        Err(LoginError::UsernameDoesntExist)?;
    }
    let user = user.unwrap();

    if existing_passkeys.as_ref().map_or(0, |v| v.len()) == 0 {
        Err(LoginError::UserHasNoCredentials)?;
    }
    let existing_passkeys = existing_passkeys.unwrap();

    // Start the authentication attempt
    let (request_challenge_response, passkey_authentication) = webauthn.start_passkey_authentication(&existing_passkeys)?;

    // Stash the authentication
    session.set_passkey_authentication_state(
        PasskeyAuthenticationState::new(user.id, passkey_authentication)).await?;

    // Send the challenge back to the client
    Ok(Json(request_challenge_response.into()))
}

async fn login_finish(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    Json(public_key_credential): Json<PublicKeyCredential>,
) -> Result<Json<User>, ServerError<Nothing>> {
    // Get the challenge from the session
    let PasskeyAuthenticationState {user_id, passkey_authentication } = session
        .take_passkey_authentication_state()
        .await?
        .ok_or(unauthorized_error!("Current session doesn't contain a PasskeyAuthenticationState. Client error or replay attack?"))?;

    // Attempt to complete the passkey authentication with the provided public key
    let authentication_result = webauthn.finish_passkey_authentication(
        &public_key_credential, &passkey_authentication)?;
    
    // At this point the autnetication has succeeded but there are a few more checks and updates we need to make
    let user = conn.interact(move |conn| {
        // TODO: perhaps this code should be moved into the model to avoid such low level code in the app
        use exemplar::Model;

        // Need a transaction because we're updating the credential and user and want it to rollback if either fail
        let tx = conn.transaction()?;

        let id = authentication_result.cred_id().clone().into();
        
        // Get the credential 
        // If it was deleted between start & finish this might fail and we should not proceed with the login
        let mut credential = Credential::fetch(&tx, &id)?;

        let mut dirty = false;

        // If the counter is non-zero, we have to check it
        let counter = authentication_result.counter();
        if counter > 0 {
            ensure_server!(counter > credential.counter, "Stored counter >= authentication result counter. Possible credential clone or re-use");
            credential.counter = counter;
            dirty = true;
        }

        let backup_state = authentication_result.backup_state();
        if backup_state != credential.backup_state {
            credential.backup_state = backup_state;
            dirty = true;
        }

        let backup_eligible = authentication_result.backup_eligible();
        if backup_eligible != credential.backup_eligible {
            credential.backup_eligible = backup_eligible;
            dirty = true;
        }

        let now = Utc::now();
        credential.last_used_date = Some(now);

        if dirty {
            credential.last_updated_date = now;
        }

        credential.update(&tx)?;

        let mut user = User::fetch_by_id(&tx, &credential.user_id)?;
        user.last_updated_date = now;
        user.last_login_date = Some(now);
        user.update(&tx)?;

        tx.commit()?;

        Ok::<_, ServerError<Nothing>>(user)
    }).await??;

    // Update the user state in the session so the user is logged in on furture requests
    session.set_user_state(&user).await?;

    Ok(Json(user))
}

#[allow(unused_variables)]
async fn fetch_user(
    DatabaseConnection(conn): DatabaseConnection,
    user_state: UserState,
) -> Result<Json<User>, ServerError<FetchError>>{
    let user = conn.interact(move |conn| 
        Ok::<_, ServerError<_>>(user_state.id.fetch_full_user(conn)?))
        .await??;

    Ok(Json(user))
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