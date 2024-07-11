#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Builder error: {message}")]
    Builder { message: &'static str },
}

pub trait Builder<T>: Sized {
    fn build(self) -> Result<T, Error>;
}

mod platform;
pub use platform::*;

pub mod peer_connector;

mod peer_map;
pub use peer_map::*;

pub mod signalling_client;

mod types;
pub use types::*;
