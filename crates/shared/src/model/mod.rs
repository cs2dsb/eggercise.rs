mod user;
pub use user::*;

#[cfg(feature = "backend")]
mod credential;
#[cfg(feature = "backend")]
pub use credential::*;

use crate::api::error::ValidationError;

pub mod constants;

mod temporary_login;
pub use temporary_login::*;

pub trait ValidateModel {
    fn validate(&self) -> Result<(), ValidationError>;
}
