use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(name = "eggercise server")]
pub struct Cli {
    #[clap(long, env, default_value = "assets")]
    pub assets_dir: PathBuf,
    #[clap(long, env, default_value = "egg.sqlite")]
    pub sqlite_connection_string: String,
    #[clap(long, env, default_value = "64")]
    pub database_command_channel_bound: usize,
    #[clap(long, env, default_value = "8080")]
    pub port: u16,
    #[clap(long, env, default_value = "127.0.0.1")]
    pub bind_addr: String,
    #[clap(long, env, default_value = "false")]
    pub secure_sessions: bool,
    #[arg(long, env, default_value = "http://localhost:8080")]
    pub webauthn_origin: String,
    #[arg(long, env, default_value = "http://localhost:8080")]
    pub cors_origin: String,
    #[arg(long, env, default_value = "localhost")]
    pub webauthn_id: String,
    #[arg(long, env, default_value = "30")]
    pub session_expiry_days: i64,
    #[arg(long, env, default_value = "10")]
    pub temporary_login_expiry_minutes: u64,
    #[clap(long, env, default_value = "egg_key.pem")]
    pub private_key_path: String,
    #[clap(long, env, default_value = "egg_key.pub.pem")]
    pub public_key_path: String,

    /// Deletes the database before starting the main program for debug purposes
    #[arg(long, env, default_value = "false")]
    pub debug_delete_database: bool,
}
