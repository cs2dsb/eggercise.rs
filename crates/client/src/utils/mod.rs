use std::{any::type_name, error::Error};

use leptos::logging::error;
use shared::api::error::FrontendError;
use wasm_bindgen::JsValue;

pub mod browser;
pub mod location;
pub mod rtc;
pub mod websocket;

mod wrap_callback;
pub use wrap_callback::*;

pub trait JsValueIntoOk<R, E>: Sized
where
    Self: Into<R>,
    E: Error,
{
    fn ok(self) -> Result<R, FrontendError<E>>;
}

impl<J, R, E> JsValueIntoOk<R, E> for J
where
    J: Into<R> + AsRef<JsValue>,
    E: Error,
{
    fn ok(self) -> Result<R, FrontendError<E>> {
        let jsvalue: &JsValue = self.as_ref();
        if jsvalue.is_null() || jsvalue.is_undefined() {
            let inner = format!(
                "Failed to convert {} to {} because it was null or undefined",
                type_name::<Self>(),
                type_name::<R>()
            );
            error!("{inner}");
            Err(FrontendError::Js { inner })
        } else {
            Ok(self.into())
        }
    }
}
