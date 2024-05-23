use serde::{Deserialize, Serialize};
use thiserror::Error;
#[cfg(feature = "backend")]
use {crate::api::error::ServerError, http::StatusCode};

use super::error::Nothing;

macro_rules! response_error {
    ($name:ident {
        $(
            #[code($variant_code:expr)]
            $variant:ident
            $({ $($var_struct_body_tt:tt)* })?
        ,)*
    }) => {

        #[derive(Debug, Clone, Serialize, Deserialize, Error)]
        pub enum $name {
            $(
                #[error("{}::{}: {:?}", stringify!($name), stringify!($variant), self)]
                $variant $({
                    $($var_struct_body_tt)*
                })?,
            )*
        }

        #[cfg(feature="backend")]
        impl From<$name> for ServerError<$name> {
            fn from(inner: $name) -> Self {
                let code = match &inner {
                    $( $name::$variant { .. } => $variant_code, )*
                };
                Self::Inner { code, inner }
            }
        }
    };
}

response_error!(RegisterError {
    #[code(StatusCode::UNAUTHORIZED)]
    UsernameUnavailable,
    #[code(StatusCode::BAD_REQUEST)]
    UsernameInvalid { message: String },
});

response_error!(LoginError {
    #[code(StatusCode::UNAUTHORIZED)]
    UsernameDoesntExist,
    #[code(StatusCode::BAD_REQUEST)]
    UsernameInvalid { message: String },
    #[code(StatusCode::INTERNAL_SERVER_ERROR)]
    UserHasNoCredentials,
});

// Alias used to allow future expansion of the errors without having to go back
// and update all routes that use it
pub type FetchError = Nothing;

response_error!(TemporaryLoginError {
    #[code(StatusCode::BAD_REQUEST)]
    AlreadyExists,
});