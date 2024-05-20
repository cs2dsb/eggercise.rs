use std::{fmt, ops::{Deref, DerefMut}};

use http::StatusCode;
use serde::{Deserialize, Serialize};

#[cfg(feature="frontend")]
pub use frontend::*;

use crate::model::ValidateModel;

#[cfg(feature="frontend")]
mod frontend {
    use std::{fmt::{self, Display}, ops::Deref};

    use super::{ErrorContext, ValidationError, WrongContentTypeError};
    use leptos::IntoView;
    use serde::{Deserialize, Serialize};
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

    #[derive(Debug, Clone, Serialize, Deserialize, Error)]
    pub struct Nothing {}

    impl fmt::Display for Nothing {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "()")
        }
    }

    #[derive(Debug, Clone, Error)]
    pub struct FrontendErrorOnly (FrontendError<Nothing>);

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
            Self::Client { message: format!("gloo-net error: {}", value.to_string()) }
        }
    }

    impl<T: Display> From<ValidationError> for FrontendError<T> {
        fn from(inner: ValidationError) -> Self {
            Self::Validation { inner }
        }
    }

    impl<T: Display> From<WrongContentTypeError> for FrontendError<T> {
        fn from(inner: WrongContentTypeError) -> Self {
            Self::WrongContentType { inner }
        }
    }

    impl <T: Display> From<JsValue> for FrontendError<T> {
        fn from(value: JsValue) -> Self {
            Self::Js { inner: JsError::from(value).to_string() }
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

    impl<T: Display> IntoView for FrontendError<T> {
        fn into_view(self) -> leptos::View {
            todo!()
        }
    }

    impl IntoView for FrontendErrorOnly {
        fn into_view(self) -> leptos::View {
            todo!()
        }
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
        write!(f, "Wrong content type, expected {} but got {:?}. Body: {}", self.expected, self.got, self.body)
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
pub struct NoValidation<T>(
    pub T
);

impl<T: Serialize> Serialize for NoValidation<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer 
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

#[derive(Debug, Clone)]
pub struct ServerError {
    pub code: StatusCode,
    pub message: String,
}

impl ServerError {
    pub fn new<S: Into<String>>(code: StatusCode, message: S) -> Self {
        ServerError {
            code,
            message: message.into(),
        }
    }
}

// impl<E> From<E> for ServerError
// where
//     E: Into<Box<dyn std::error::Error>>,
// {
//     fn from(err: E) -> Self {
//         Self::new(
//             StatusCode::INTERNAL_SERVER_ERROR,
//             format!("Something went wrong: {:?}", err.into()),
//         )
//     }
// }

impl From<anyhow::Error> for ServerError {
    fn from(err: anyhow::Error) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {:?}", err),
        )
    }
}

#[cfg(feature="backend")]
impl From<deadpool_sqlite::InteractError> for ServerError {
    fn from(err: deadpool_sqlite::InteractError) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {:?}", err),
        )
    }
}

#[cfg(feature="backend")]
impl From<webauthn_rs::prelude::WebauthnError> for ServerError {
    fn from(err: webauthn_rs::prelude::WebauthnError) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {:?}", err),
        )
    }
}