use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(name = "eggercise server")]
pub struct Cli {
    /// Path to the root of the web server assets dir
    #[clap(long, env, default_value = "assets")]
    pub assets_dir: PathBuf,

    /// SQLite file path
    #[clap(long, env, default_value = "egg.sqlite")]
    pub sqlite_connection_string: String,

    /// Port to run on
    #[clap(long, env, default_value = "8080")]
    pub port: u16,

    /// IP to bind to
    #[clap(long, env, default_value = "127.0.0.1")]
    pub bind_addr: String,

    /// Should session cookies be secure or not
    #[clap(long, env, default_value = "false")]
    pub secure_sessions: bool,

    /// Origin url for Webauthn
    #[arg(long, env, default_value = "http://localhost:8080")]
    pub webauthn_origin: String,

    /// Origin url for CORS
    #[arg(long, env, default_value = "http://localhost:8080")]
    pub cors_origin: String,

    /// Webauthn ID needs to match the domain
    #[arg(long, env, default_value = "localhost")]
    pub webauthn_id: String,

    /// Session cookie expiry time in days
    #[arg(long, env, default_value = "30")]
    pub session_expiry_days: i64,

    /// Expiry time for the temporary login QR codes
    #[arg(long, env, default_value = "10")]
    pub temporary_login_expiry_minutes: u64,

    /// Path to the private key used for push notifications
    #[clap(long, env, default_value = "egg_key.pem")]
    pub private_key_path: String,

    /// Path to the public key used for push notifications
    #[clap(long, env, default_value = "egg_key.pub.pem")]
    pub public_key_path: String,

    /// Enables tracing span events, mainly useful for timing spans
    #[clap(long, env, default_value = "false")]
    pub log_span_events: bool,

    /// Deletes the database before starting the main program for debug purposes
    #[arg(long, env, default_value = "false")]
    pub debug_delete_database: bool,
}
