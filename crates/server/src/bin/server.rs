#![feature(duration_constructors)]

use std::{
    fs::{read_to_string, remove_file, File},
    io::Read,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use axum::{
    extract::{MatchedPath, Request},
    http::{HeaderName, HeaderValue, Method, StatusCode, Uri},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use base64::prelude::{Engine as _, BASE64_URL_SAFE};
use chrono::Utc;
use clap::Parser;
use client::ROUTE_URLS;
use deadpool_sqlite::{Config, Hook, Runtime};
use futures::future::join_all;
use server::{
    cli::Cli,
    db,
    middleware::{CsrfLayer, RegenerateToken},
    routes::{
        auth::*,
        notifications::{remove_push_subscription, update_push_subscription, vapid},
        ping::ping,
        websocket::websocket_handler,
    },
    AppError, AppState, VapidPrivateKey, VapidPubKey,
};
use shared::{
    api::{
        error::{Nothing, ServerError},
        payloads::Notification,
        Auth, Object, CSRF_HEADER,
    },
    configure_tracing, load_dotenv,
    model::{PushNotificationSubscription, User},
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    classify::ServerErrorsFailureClass,
    compression::CompressionLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tower_sessions::{cookie::time::Duration as CookieDuration, Expiry, SessionManagerLayer};
use tower_sessions_deadpool_sqlite_store::DeadpoolSqliteStore;
use tracing::{debug, error, info, info_span, Span};
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, Urgency, VapidSignatureBuilder,
    WebPushClient, WebPushError, WebPushMessageBuilder,
};
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
    let (ran, new_version) =
        db::run_migrations(&args.sqlite_connection_string, env!("CARGO_PKG_VERSION"))?;
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

    let vapid_pub_key: VapidPubKey = {
        let base64_key = read_to_string(&args.public_key_path)?;
        BASE64_URL_SAFE
            .decode(&base64_key)
            .context(format!("Decoding {}", args.public_key_path))?
            .into()
    };

    let vapid_private_key: VapidPrivateKey = {
        let mut file = File::open(&args.private_key_path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        bytes.into()
    };

    // Grab a connection before we move the pool
    let notifier_db_connection = pool.get().await?;
    let notifier_private_key = vapid_private_key.clone();

    let state = AppState {
        pool,
        webauthn,
        args: Arc::new(args.clone()),
        vapid_pub_key,
        vapid_private_key,
        websocket_clients: Default::default(),
    };

    // Map all routes the client can handle to the index.html
    let client_routes = {
        let mut router = Router::new();
        for path in ROUTE_URLS.iter().filter(|p| **p != "/") {
            router = router.nest_service(path, ServeFile::new(args.assets_dir.join("index.html")));
        }
        router
    };

    if let Some(new_version) = new_version {
        // Double task is just to display any panics in the inner task
        tokio::spawn(async move {
            let join_handle = tokio::spawn(async move {
                let notify_users = notifier_db_connection
                    .interact(move |conn| User::fetch_all_with_push_notifications_enabled(conn))
                    .await??;

                if notify_users.len() == 0 {
                    return Ok(());
                }
                let keyref = &notifier_private_key;
                let new_version = &new_version;
                let results = join_all(notify_users.iter().map(|user| async move {
                    if let Some(PushNotificationSubscription {
                        endpoint,
                        key: p256dh,
                        auth,
                    }) = user.push_notification_subscription.clone()
                    {
                        debug!(
                            "Notifying {} ({}) we just started version {}",
                            user.username, user.id, new_version,
                        );
                        let notification = Notification {
                            title: "Eggercise updated".to_string(),
                            body: Some(format!("Version {} is now available", new_version)),
                            icon: None,
                            sent: Utc::now(),
                        };
                        let message_bytes = serde_json::to_vec(&notification)?;

                        let subscription_info = SubscriptionInfo::new(endpoint, p256dh, auth);

                        let sig_builder =
                            VapidSignatureBuilder::from_pem(keyref.cursor(), &subscription_info)?
                                .build()?;

                        let mut message_builder = WebPushMessageBuilder::new(&subscription_info);
                        message_builder.set_payload(ContentEncoding::Aes128Gcm, &message_bytes);
                        message_builder.set_vapid_signature(sig_builder);
                        message_builder.set_urgency(Urgency::High);
                        message_builder.set_ttl(Duration::from_hours(3).as_secs() as u32);

                        let message = message_builder.build()?;

                        let client = IsahcWebPushClient::new()?;
                        if let Err(e) = client.send(message).await {
                            error!("Error sending push notification: {:?}", e);
                            Err(e)?
                        } else {
                            debug!("Push sent ok");
                        }
                    }
                    Ok::<_, WebPushError>(())
                }))
                .await
                .into_iter()
                .collect::<Vec<Result<_, _>>>();

                let (user_errors_unknown, user_errors_invalid_sub): (Vec<_>, _) = notify_users
                    .into_iter()
                    .zip(results.into_iter())
                    .filter_map(|(user, r)| r.err().map(|e| (user, e)))
                    .partition(|(_, err)| match err {
                        // TODO: there might be some other errors this behaviour should apply to
                        WebPushError::EndpointNotValid
                        | WebPushError::InvalidUri
                        | WebPushError::EndpointNotFound => false,
                        _ => true,
                    });

                if user_errors_invalid_sub.len() > 0 {
                    notifier_db_connection
                        .interact(move |conn| {
                            // TODO: move into shared db code
                            let mut stmt = conn.prepare(
                                "UPDATE User SET push_notification_subscription = NULL WHERE id = ?1",
                            )?;
                            for (user, _) in user_errors_invalid_sub.iter() {
                                stmt.execute(&[&user.id])?;
                                // TODO: send a message to the service worker to resubscribe
                                error!(
                                    "User {} ({}) had an invalid subscription. It's been removed",
                                    user.username, user.id
                                );
                            }
                            Ok::<_, rusqlite::Error>(())
                        })
                        .await??;
                }

                if user_errors_unknown.len() > 0 {
                    let message = user_errors_unknown
                        .into_iter()
                        .map(|(user, err)| format!("(user_id: {}, {:?})", user.id, err))
                        .collect::<Vec<_>>()
                        .join(",");
                    Err(ServerError::<Nothing>::Other {
                        message,
                    })
                } else {
                    Ok(())
                }
            });
            match join_handle.await {
                Err(e) => error!("Join error in notifier task: {e}"),
                Ok(r) => {
                    if let Err(e) = r {
                        error!("Error from notifier task: {e}");
                    }
                }
            }
        });
    }

    let listener = TcpListener::bind(socket).await?;
    debug!("listening on {}", listener.local_addr()?);

    axum::serve(
        listener,
        Router::new()
            .merge(client_routes)
            // User/auth routes
            .route(Auth::RegisterStart.path(), post(register_start))
            .route(Auth::RegisterFinish.path(), post(register_finish))
            .route(Auth::LoginStart.path(), post(login_start))
            .route(Auth::LoginFinish.path(), post(login_finish))
            .route(
                Auth::RegisterNewKeyStart.path(),
                post(register_new_key_start),
            )
            .route(
                Auth::RegisterNewKeyFinish.path(),
                post(register_new_key_finish),
            )
            .route(Auth::TemporaryLogin.path(), get(temporary_login))
            .route(Object::User.path(), get(fetch_user))
            .route(
                Auth::CreateTemporaryLogin.path(),
                post(create_temporary_login),
            )
            .route(Object::QrCodeId.path(), get(generate_qr_code))
            // Notification routes
            .route(Object::Vapid.path(), get(vapid))
            .route(
                Object::PushSubscription.path(),
                post(update_push_subscription).delete(remove_push_subscription),
            )
            .route(Object::Ping.path(), get(ping))
            .route(Object::Websocket.path(), get(websocket_handler))
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
                CsrfLayer::new()
                    .regenerate(RegenerateToken::PerSession)
                    .request_header(CSRF_HEADER)
                    .response_header(CSRF_HEADER),
            )
            .layer(
                ServiceBuilder::new()
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(|request: &Request<_>| {
                                let span = info_span!(
                                    "http_log",
                                    status = tracing::field::Empty,
                                    method = ?request.method(),
                                    matched_path = tracing::field::Empty,
                                    path = tracing::field::Empty,
                                );

                                if let Some(matched_path) = request
                                    .extensions()
                                    .get::<MatchedPath>()
                                    .map(MatchedPath::as_str)
                                {
                                    span.record("matched_path", matched_path);
                                } else {
                                    // Fallback if the path isn't matched
                                    span.record("path", request.uri().to_string());
                                }

                                span
                            })
                            .on_response(|response: &Response, _latency: Duration, span: &Span| {
                                span.record("status", response.status().to_string());
                            })
                            .on_failure(
                                |error: ServerErrorsFailureClass,
                                 _latency: Duration,
                                 span: &Span| {
                                    if let ServerErrorsFailureClass::StatusCode(code) = error {
                                        span.record("status", code.to_string());
                                    }
                                },
                            ),
                    )
                    .layer(
                        SessionManagerLayer::new(session_store)
                            .with_secure(args.secure_sessions)
                            .with_expiry(Expiry::OnInactivity(CookieDuration::days(
                                args.session_expiry_days,
                            ))),
                    )
                    .layer(SetResponseHeaderLayer::if_not_present(
                        HeaderName::from_static("cross-origin-opener-policy"),
                        HeaderValue::from_static("same-origin"),
                    ))
                    .layer(SetResponseHeaderLayer::if_not_present(
                        HeaderName::from_static("cross-origin-embedder-policy"),
                        HeaderValue::from_static("require-corp"),
                    ))
                    .layer(
                        CorsLayer::new()
                            .allow_methods([Method::GET, Method::POST, Method::DELETE])
                            .allow_origin(args.cors_origin.parse::<HeaderValue>()?),
                    ),
            )
            .layer(CompressionLayer::new())
            .with_state(state)
            .into_make_service_with_connect_info::<SocketAddr>(),
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
