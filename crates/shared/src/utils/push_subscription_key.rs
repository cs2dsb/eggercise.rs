use std::fmt::Display;

use base64::prelude::{Engine as _, BASE64_URL_SAFE};
use web_sys::{js_sys::Uint8Array, PushEncryptionKeyName, PushSubscription};

use crate::api::error::FrontendError;

pub fn get_subscription_key<T: Display>(
    sub: &PushSubscription,
    key: PushEncryptionKeyName,
) -> Result<String, FrontendError<T>> {
    let key = sub.get_key(key)?.ok_or(FrontendError::Js {
        inner: format!(
            "Browser didn't return a {:?} key for an established subscription",
            key
        ),
    })?;

    let bytes: Vec<u8> = Uint8Array::new(&key).to_vec();

    Ok(BASE64_URL_SAFE.encode(bytes))
}
