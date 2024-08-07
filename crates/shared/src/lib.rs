#![feature(macro_metavar_expr)]

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::fmt::format::FmtSpan;
#[cfg(feature = "build")]
use {
    cargo_toml::Manifest,
    chrono::Utc,
    std::{path::Path, str::FromStr},
};

pub mod api;
pub mod model;
pub mod rtc;
pub mod types;
pub mod utils;

pub fn configure_tracing(span_events: bool) -> Result<(), SetGlobalDefaultError> {
    let mut builder = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env());

    if span_events {
        builder = builder.with_span_events(FmtSpan::CLOSE)
    }

    let subscriber = builder.finish();

    tracing::subscriber::set_global_default(subscriber)
}

pub fn load_dotenv() -> Result<Option<PathBuf>, dotenv::Error> {
    match dotenv::dotenv() {
        // Swallow NotFound error since the .env is optional
        Err(dotenv::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        r => r.map(|p| Some(p)),
    }
}

#[cfg(feature = "build")]
fn read_manifest<P: AsRef<Path>>(path: P) -> Result<Manifest, cargo_toml::Error> {
    Ok(Manifest::from_path(path)?)
}

#[cfg(feature = "build")]
#[derive(Debug, Clone)]
pub struct CrateInfo {
    pub manifest_dir: PathBuf,
    pub lib_file_name: String,
    pub package_name: String,
    pub version: String,
    pub version_with_timestamp: String,
}

#[cfg(feature = "build")]
fn get_crate_info<P: AsRef<Path>>(crate_path: P) -> Result<CrateInfo, anyhow::Error> {
    let shared_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR"))?;
    let manifest_dir = shared_dir.join(crate_path.as_ref());
    let manifest = read_manifest(manifest_dir.join("Cargo.toml"))?;
    let package =
        manifest.package.ok_or(anyhow::anyhow!("Worker manifest missing package entry"))?;
    let lib_file_name = package.name.replace("-", "_");
    let version = package.version().to_string();
    let version_with_timestamp = format!("{}_{}", version, Utc::now().format("%Y%m%d%H%M%S"),);
    let package_name = package.name;

    Ok(CrateInfo { manifest_dir, lib_file_name, package_name, version, version_with_timestamp })
}

#[cfg(feature = "build")]
pub fn get_service_worker_info() -> Result<CrateInfo, anyhow::Error> {
    get_crate_info("../service-worker")
}

#[cfg(feature = "build")]
pub fn get_client_info() -> Result<CrateInfo, anyhow::Error> {
    get_crate_info("../client")
}

#[cfg(feature = "build")]
pub fn get_server_info() -> Result<CrateInfo, anyhow::Error> {
    get_crate_info("../server")
}

pub const SERVICE_WORKER_PACKAGE_FILENAME: &str = "service_worker_package.json";
pub const SERVICE_WORKER_PACKAGE_URL: &str = "/wasm/service_worker_package.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct HashedFile {
    pub path: String,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceWorkerPackage {
    pub version: String,
    pub files: Vec<HashedFile>,
}

impl ServiceWorkerPackage {
    pub fn file<'a>(&'a self, path: &str) -> Option<&'a HashedFile> {
        self.files.iter().find(|f| f.path.as_str() == path)
    }
}
