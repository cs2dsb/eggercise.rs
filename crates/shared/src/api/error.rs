use std::{
    fmt,
    ops::{Deref, DerefMut},
};

#[cfg(feature = "wasm")]
pub use frontend::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[cfg(feature = "backend")]
use {
    axum::{
        response::{IntoResponse, Response},
        Json,
    },
    std::fmt::Display,
};
#[cfg(any(feature = "backend", feature = "wasm"))]
use {http::StatusCode, std::error::Error};

use crate::model::ValidateModel;

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub struct Nothing {}

impl fmt::Display for Nothing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "()")
    }
}

#[cfg(feature = "wasm")]
mod frontend {
    use std::{
        any::type_name,
        fmt::{self, Display},
        ops::Deref,
    };

    use leptos::{view, IntoView};
    use thiserror::Error;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::js_sys::{
        Error as GenericJsError,
        RangeError as JsRangeError,
        ReferenceError as JsReferenceError,
        SyntaxError as JsSyntaxError,
        // TryFromIntError as JsTryFromIntError,
        TypeError as JsTypeError,
        UriError as JsUriError,
    };

    use super::{ErrorContext, Nothing, ValidationError, WrongContentTypeError};

    #[derive(Debug, Clone, Error)]
    pub enum JsError {
        #[error("GenericJs Error: {0:?}")]
        GenericJs(GenericJsError),
        #[error("JsRange Error: {0:?}")]
        JsRange(JsRangeError),
        #[error("JsReference Error: {0:?}")]
        JsReference(JsReferenceError),
        #[error("JsSyntax Error: {0:?}")]
        JsSyntax(JsSyntaxError),
        // #[error("JsTryFromInt Error: {0:?}")]
        // JsTryFromInt(JsTryFromIntError),
        #[error("JsType Error: {0:?}")]
        JsType(JsTypeError),
        #[error("JsUri Error: {0:?}")]
        JsUri(JsUriError),
        #[error("UnknownJsValue Error: {0:?}")]
        UnknownJsValue(String),
    }

    impl From<JsValue> for JsError {
        fn from(err: JsValue) -> JsError {
            if err.is_instance_of::<JsRangeError>() {
                return JsError::JsRange(err.into());
            }
            if err.is_instance_of::<JsReferenceError>() {
                return JsError::JsReference(err.into());
            }
            if err.is_instance_of::<JsSyntaxError>() {
                return JsError::JsSyntax(err.into());
            }
            // Not supported by JsCast
            // if err.is_instance_of::<JsTryFromIntError>() {
            //     return JsError::JsTryFromInt(err.into());
            // }
            if err.is_instance_of::<JsTypeError>() {
                return JsError::JsType(err.into());
            }
            if err.is_instance_of::<JsUriError>() {
                return JsError::JsUri(err.into());
            }
            if err.is_instance_of::<GenericJsError>() {
                return JsError::GenericJs(err.into());
            }
            JsError::UnknownJsValue(format!("{:?}", err))
        }
    }

    #[derive(Debug, Clone, Error)]
    pub enum FrontendError<T: Display> {
        #[error("{inner}")]
        Inner { inner: T },
        #[error("{message}")]
        Client { message: String },
        #[error("{inner}")]
        Js { inner: String },
        #[error("{inner}")]
        Validation { inner: ValidationError },
        #[error("{inner}")]
        WrongContentType { inner: WrongContentTypeError },
        #[error("{inner}\nContext: {context}")]
        WithContext { context: String, inner: Box<Self> },
    }

    impl<T: Display> FrontendError<T> {
        pub fn map_display(inner: T) -> Self {
            Self::Inner {
                inner,
            }
        }
    }

    #[derive(Debug, Clone, Error)]
    pub struct FrontendErrorOnly(FrontendError<Nothing>);

    impl fmt::Display for FrontendErrorOnly {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt(f)
        }
    }

    impl Deref for FrontendErrorOnly {
        type Target = FrontendError<Nothing>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl From<FrontendError<Nothing>> for FrontendErrorOnly {
        fn from(value: FrontendError<Nothing>) -> Self {
            Self(value)
        }
    }

    impl<T: Display> From<gloo_net::Error> for FrontendError<T> {
        fn from(value: gloo_net::Error) -> Self {
            Self::Client {
                message: format!("gloo-net error: {}", value.to_string()),
            }
        }
    }

    impl<T: Display> From<ValidationError> for FrontendError<T> {
        fn from(inner: ValidationError) -> Self {
            Self::Validation {
                inner,
            }
        }
    }

    impl<T: Display> From<WrongContentTypeError> for FrontendError<T> {
        fn from(inner: WrongContentTypeError) -> Self {
            Self::WrongContentType {
                inner,
            }
        }
    }

    impl<T: Display> From<JsValue> for FrontendError<T> {
        fn from(value: JsValue) -> Self {
            Self::Js {
                inner: JsError::from(value).to_string(),
            }
        }
    }

    impl<T: Display, E: Into<FrontendError<T>>> ErrorContext<FrontendError<T>> for E {
        fn with_context<S: Into<String>, F: FnOnce() -> S>(self, context: F) -> FrontendError<T> {
            self.context(context())
        }
        fn context<S: Into<String>>(self, context: S) -> FrontendError<T> {
            FrontendError::WithContext {
                context: context.into(),
                inner: Box::new(self.into()),
            }
        }
    }

    impl<T: Display> IntoView for &FrontendError<T> {
        fn into_view(self) -> leptos::View {
            use FrontendError::*;
            match self {
                Inner { inner } => {
                    let name = type_name::<T>();
                    view! { <li>{ format!("{name} Error:\n{inner}") }</li> }.into_view()
                },
                Client { message } => view! { <li>{ format!("ClientError:\n{message}") }</li> }.into_view(),
                Js { inner } => view! { <li>{ format!("JsError:\n{inner}") }</li> }.into_view(),
                Validation { inner } => {
                    let errors = inner.error_messages.join(", ");
                    view! { <li>{ format!("ValidationErrors:\n{errors}") }</li> }.into_view()
                },
                WrongContentType { inner: WrongContentTypeError { expected, got, body } } => {
                    view! { <li>{ format!("WrongContentTypeError:\nExpected: {expected}\nGot:{:?}\nBody: {body}", got) }</li> }.into_view()
                },
                WithContext { context, inner } => {
                    let inner_view = inner.into_view();
                    view! { <li>{ format!("{context}\n") } <ul class="error-list">{ inner_view }</ul></li> }.into_view()
                },
            }
        }
    }

    impl IntoView for &FrontendErrorOnly {
        fn into_view(self) -> leptos::View {
            todo!()
        }
    }
}

#[cfg(any(feature = "backend", feature = "wasm"))]
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[must_use]
pub enum ServerError<T: Error> {
    #[error("ServerError::Inner{{ code: {code}, inner: {inner} }}")]
    Inner {
        #[serde(with = "http_serde::status_code")]
        code: StatusCode,
        inner: T,
    },

    #[error("ServerError::Unauthorized {{ {message} }}")]
    Unauthorized { message: String },

    #[error("ServerError::Status {{ {message} }}")]
    Status {
        #[serde(with = "http_serde::status_code")]
        code: StatusCode,
        message: String,
    },

    // TODO: do these extra variants with the same inner type actually add anything above
    // prefixing       the message with the name of the origin type?
    #[error("ServerError::Json {{ {message} }}")]
    Json { message: String },

    #[error("ServerError::Database {{ {message} }}")]
    Database { message: String },

    #[error("ServerError::Deadpool {{ {message} }}")]
    Deadpool { message: String },

    #[error("ServerError::Webauthn {{ {message} }}")]
    Webauthn { message: String },

    #[error("ServerError::Other{{ {message} }}")]
    Other { message: String },

    #[error("{inner}\nWithContext: {context}")]
    WithContext { context: String, inner: Box<Self> },
}

#[cfg(feature = "backend")]
#[macro_export]
macro_rules! other_error {
    ($($t:tt)*) => (ServerError::Other{ message: format_args!($($t)*).to_string() })
}

#[cfg(feature = "backend")]
#[macro_export]
macro_rules! ensure_server {
    ($expr:expr, $($t:tt)*) => (if $expr {
        Err(ServerError::Other{ message: format!("Assertion failed: {}", format_args!($($t)*)) })?
    })
}

#[cfg(feature = "backend")]
#[macro_export]
macro_rules! unauthorized_error {
    ($($t:tt)*) => (ServerError::Unauthorized{ message: format_args!($($t)*).to_string() })
}

#[cfg(feature = "backend")]
#[macro_export]
macro_rules! bad_request_error {
    ($($t:tt)*) => (ServerError::Status{ code: StatusCode::BAD_REQUEST, message: format_args!($($t)*).to_string() })
}

#[cfg(feature = "backend")]
#[macro_export]
macro_rules! status_code_error {
    ($code:expr, $($t:tt)*) => (ServerError::Status{ code: $code, message: format_args!($($t)*).to_string() })
}

#[cfg(feature = "backend")]
impl<T: Error, E: Into<ServerError<T>>> ErrorContext<ServerError<T>> for E {
    fn with_context<S: Into<String>, F: FnOnce() -> S>(self, context: F) -> ServerError<T> {
        self.context(context())
    }
    fn context<S: Into<String>>(self, context: S) -> ServerError<T> {
        ServerError::WithContext {
            context: context.into(),
            inner: Box::new(self.into()),
        }
    }
}

#[cfg(feature = "backend")]
impl<T: Error> From<rusqlite::Error> for ServerError<T> {
    fn from(value: rusqlite::Error) -> Self {
        Self::Database {
            message: value.to_string(),
        }
    }
}

#[cfg(feature = "backend")]
impl<T: Error> From<serde_json::Error> for ServerError<T> {
    fn from(value: serde_json::Error) -> Self {
        Self::Json {
            message: value.to_string(),
        }
    }
}

#[cfg(feature = "backend")]
impl<T: Error> From<deadpool_sqlite::InteractError> for ServerError<T> {
    fn from(value: deadpool_sqlite::InteractError) -> Self {
        Self::Deadpool {
            message: value.to_string(),
        }
    }
}

#[cfg(feature = "backend")]
impl<T: Error> From<webauthn_rs::prelude::WebauthnError> for ServerError<T> {
    fn from(value: webauthn_rs::prelude::WebauthnError) -> Self {
        Self::Webauthn {
            message: value.to_string(),
        }
    }
}

#[cfg(feature = "backend")]
impl<T: Error> ServerError<T> {
    fn code(&self) -> StatusCode {
        use ServerError::*;

        match &self {
            Inner {
                code, ..
            } => code.to_owned(),
            Unauthorized {
                ..
            } => StatusCode::UNAUTHORIZED,
            Status {
                code, ..
            } => code.to_owned(),
            WithContext {
                inner, ..
            } => inner.code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "backend")]
impl<T: Error + Serialize> IntoResponse for ServerError<T> {
    fn into_response(self) -> Response {
        let code = self.code();
        let json = Json(self);
        (code, json).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrongContentTypeError {
    pub expected: String,
    pub got: Option<String>,
    pub body: String,
}

impl fmt::Display for WrongContentTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Wrong content type, expected {} but got {:?}. Body: {}",
            self.expected, self.got, self.body
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub error_messages: Vec<String>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Validation error(s):")?;
        for e in self.error_messages.iter() {
            writeln!(f, "   {e}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NoValidation<T>(pub T);

impl<T: Serialize> Serialize for NoValidation<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T> Deref for NoValidation<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for NoValidation<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> ValidateModel for NoValidation<T> {
    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl ValidateModel for () {
    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }
}

#[cfg(feature = "backend")]
pub trait ServerErrorContext<T, X, Y>: Sized {
    fn with_context<S: Display, F: FnOnce() -> S>(self, f: F) -> Result<X, Y> {
        self.context(f())
    }
    fn context<S: Display>(self, context: S) -> Result<X, Y>;
}

#[cfg(feature = "backend")]
impl<R, T: Error, E: Into<ServerError<T>>> ServerErrorContext<T, R, ServerError<T>> for Result<R, E>
where
    E: Into<ServerError<T>>,
{
    fn context<S: Display>(self, context: S) -> Result<R, ServerError<T>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let inner = Box::new(e.into());
                Err(ServerError::WithContext {
                    context: context.to_string(),
                    inner,
                })
            }
        }
    }
}

pub trait ErrorContext<E>: Sized {
    /// Add helpful context to errors
    ///
    /// Backtrace will be captured  if nightly feature is enabled
    ///
    /// `context` is provided as a closure to avoid potential formatting cost if
    /// the result isn't an error
    #[allow(dead_code)]
    fn with_context<S: Into<String>, F: FnOnce() -> S>(self, context: F) -> E;
    /// Add helpful context to errors
    ///
    /// Backtrace will be captured  if nightly feature is enabled
    ///
    /// `context` is provided as a closure to avoid potential formatting cost if
    /// the result isn't an error
    fn context<S: Into<String>>(self, context: S) -> E;
}

pub trait ResultContext<T, E: ErrorContext<E>> {
    fn with_context<S: Into<String>, F: FnOnce() -> S>(self, context: F) -> Result<T, E>;
    fn context<S: Into<String>>(self, context: S) -> Result<T, E>;
}

impl<T, E: ErrorContext<E>> ResultContext<T, E> for Result<T, E> {
    fn with_context<S: Into<String>, F: FnOnce() -> S>(self, context: F) -> Result<T, E> {
        self.context(context())
    }
    fn context<S: Into<String>>(self, context: S) -> Result<T, E> {
        self.map_err(|e| e.context(context))
    }
}
