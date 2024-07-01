use tracing::trace;
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys::Uint8Array, PushManager, PushSubscription, PushSubscriptionOptionsInit};

use crate::api::{
    error::{FrontendError, Nothing, ServerError},
    fetch_fns::notifications::{fetch_vapid, update_subscription},
};

pub async fn record_subscription(
    push_manager: &PushManager,
) -> Result<(), FrontendError<ServerError<Nothing>>> {
    // Get the servers public key (it's not base64 encoded so no conversion is
    // needed)
    let vapid = fetch_vapid().await?;
    trace!("Got vapid: {:?}", vapid.key);

    let key = Uint8Array::from(&vapid.key[..]);

    let mut options = PushSubscriptionOptionsInit::new();
    options.user_visible_only(true);
    options.application_server_key(Some(&key));

    let subscription: PushSubscription =
        JsFuture::from(push_manager.subscribe_with_options(&options)?)
            .await?
            .into();

    // Pass the sub to the backend
    update_subscription(&subscription).await?;

    Ok(())
}
