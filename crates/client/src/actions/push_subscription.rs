use leptos::{create_action, Action, SignalUpdate, Trigger, WriteSignal};
use shared::api::{
    browser::record_subscription, error::FrontendError,
    fetch_fns::notifications::remove_subscription,
};
use tracing::debug;
use wasm_bindgen_futures::JsFuture;
use web_sys::PushSubscription;

use crate::utils::browser::get_web_push_manger;

pub fn subscribe_action(
    action_updated_subscription_trigger: Trigger,
    set_error: WriteSignal<Option<String>>,
) -> Action<(), ()> {
    create_action(move |_| {
        debug!("Subscribing to push notification");

        // Clear the error immediately so if there was a previous one the user action
        // that kicked this off doesn't appear to have done nothing/immediately
        // failed
        set_error.update(|v| *v = None);

        async move {
            let result = async move {
                let push_manager = get_web_push_manger().await?;
                record_subscription(&push_manager).await
            }
            .await;

            if let Err(e) = result {
                set_error.update(|v| *v = Some(e.to_string()));
            } else {
                action_updated_subscription_trigger.notify();
            }
        }
    })
}

pub fn unsubscribe_action(
    action_updated_subscription_trigger: Trigger,
    set_error: WriteSignal<Option<String>>,
) -> Action<(), ()> {
    create_action(move |_| {
        debug!("Unsubscribing from push notification");

        // Clear the error immediately so if there was a previous one the user action
        // that kicked this off doesn't appear to have done nothing/immediately
        // failed
        set_error.update(|v| *v = None);

        async move {
            let result = async move {
                let push_manager = get_web_push_manger().await?;
                let subscription: PushSubscription =
                    JsFuture::from(push_manager.get_subscription()?).await?.into();
                JsFuture::from(subscription.unsubscribe()?).await?;

                // Remove the subscription from the backend
                remove_subscription().await?;

                action_updated_subscription_trigger.notify();
                Ok::<_, FrontendError<_>>(())
            }
            .await;

            if let Err(e) = result {
                set_error.update(|v| *v = Some(e.to_string()));
            }
        }
    })
}
