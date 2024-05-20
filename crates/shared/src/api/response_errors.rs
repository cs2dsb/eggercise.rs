use serde::{Deserialize, Serialize};
use http::StatusCode;
use super::error::{
    ErrorContext,
    ServerError,
};
use std::fmt;
use thiserror::Error;

#[cfg(feature="backend")]
use axum::{ Json, response::{IntoResponse, Response}};



// Generates an enum that includes a "Server" variant for generic status+message errors 
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
            $($variant $({
                $($var_struct_body_tt)* 
            })? ,)*

            Server { 
                #[serde(with = "http_serde::status_code")]
                code: StatusCode, 
                message: String,
            },

            WithContext { context: String, inner: Box<Self> },
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match &self {
                    $(Self::$variant { .. } => write!(f, "{}::{}: {:?}", stringify!($name), stringify!($variant), self),)*
                    Self::Server { code, message } => write!(f, "{}::Server: {code}, {message}", stringify!($name)),
                    Self::WithContext { inner, context } => {
                        inner.fmt(f)?;
                        write!(f, "\nContext: {context}")
                    },
                }
            }
        }

        // impl From<ServerError> for $name {
        //     fn from(ServerError { code, message }: ServerError) -> Self {
        //         Self::Server{
        //             code,
        //             message,
        //         }
        //     }
        // }

        impl<T> From<T> for $name 
        where
            T: Into<ServerError>
        {
            fn from(err: T) -> Self {
                let ServerError { code, message } = err.into();
                Self::Server {
                    code,
                    message,
                }
            }
        }

        // #[cfg(feature="backend")]
        // impl<E> From<E> for $name
        // where
        //     E: Into<Box<dyn std::error::Error>>,
        // {
        //     fn from(err: E) -> Self {
        //         ServerError::new(
        //             StatusCode::INTERNAL_SERVER_ERROR,
        //             format!("Something went wrong: {:?}", err.into()),
        //         ).into()
        //     }
        // }

        // #[cfg(feature="backend")]
        // impl From<BackendError> for $name {
        //     fn from(err: BackendError) -> Self {
        //         ServerError::new(
        //             StatusCode::INTERNAL_SERVER_ERROR,
        //             format!("Something went wrong: {:?}", err),
        //         ).into()
        //     }
        // }

        #[cfg(feature="backend")]
        impl $name {
            fn code(&self) -> StatusCode {
                match &self {
                    $(Self::$variant { .. } => $variant_code,)*
                    Self::Server { code, .. } => *code,
                    Self::WithContext { inner, .. } => inner.code(),
                }
            }
        }

        #[cfg(feature="backend")]
        impl IntoResponse for $name {
            fn into_response(self) -> Response {
                (self.code(), Json(self)).into_response()
            }
        }

        impl<E: Into<$name>> ErrorContext<$name> for E {
            fn with_context<S: Into<String>, F: FnOnce() -> S>(self, context: F) -> $name {
                self.context(context())
            }
            fn context<S: Into<String>>(self, context: S) -> $name {
                $name::WithContext {
                    context: context.into(),
                    inner: Box::new(self.into()),
                }
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

response_error!(FetchError {});