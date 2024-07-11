pub mod error;
pub mod payloads;
pub mod response_errors;

mod object;
pub use object::*;

mod auth;
pub use auth::*;

#[cfg(feature = "wasm")]
pub mod browser;
#[cfg(feature = "wasm")]
pub mod fetch_fns;

pub const API_BASE_PATH: &str = "/api/";
pub const CSRF_HEADER: &str = "X-CSRF-TOKEN";
