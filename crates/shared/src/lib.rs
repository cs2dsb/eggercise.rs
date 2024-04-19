use std::path::PathBuf;

use tracing_subscriber::fmt::format::FmtSpan;

pub fn configure_tracing() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_line_number(true)
            .with_file(true)
            //.with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
            .with_span_events(FmtSpan::CLOSE)
            .finish(),
    )
    .expect("Failed to set default tracing subscriber");
}

pub fn load_dotenv() -> Result<Option<PathBuf>, dotenv::Error> {
    match dotenv::dotenv() {
        // Swallow NotFound error since the .env is optional
        Err(dotenv::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        r => r.map(|p| Some(p)),
    }
}