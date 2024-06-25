use std::fmt::Display;

use leptos::{
    component, create_action, create_local_resource, create_signal, view, window, IntoView,
    SignalGet, SignalUpdate, SignalWith,
};
use shared::api::error::{FrontendError, Nothing};
use tracing::debug;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::Uint8Array, PushManager, PushSubscription, PushSubscriptionOptionsInit,
    ServiceWorkerRegistration,
};

use crate::api::{fetch_vapid, remove_subscription, update_subscription};

async fn get_registration<T: Display>() -> Result<ServiceWorkerRegistration, FrontendError<T>> {
    let sw = window().navigator().service_worker();
    Ok(JsFuture::from(sw.ready()?).await?.into())
}

async fn get_push_manger<T: Display>() -> Result<PushManager, FrontendError<T>> {
    Ok(get_registration().await?.push_manager()?)
}

#[component]
pub fn Notificiations() -> impl IntoView {
    let is_subscribed_resource = create_local_resource(
        || {},
        |_| async move {
            let push_manager = get_push_manger().await?;
            let maybe_subscription = JsFuture::from(push_manager.get_subscription()?).await?;
            let is_subscribed =
                !(maybe_subscription.is_null() || maybe_subscription.is_undefined());

            debug!("is_subscribed: {is_subscribed}");
            Ok::<_, FrontendError<Nothing>>(is_subscribed)
        },
    );

    let is_subscribed = move || {
        is_subscribed_resource
            .get()
            .map(|r| r.unwrap_or(false))
            .unwrap_or(false)
    };
    let (error, set_error) = create_signal(None::<String>);

    let subscribe_action = create_action(move |subscribe: &bool| {
        let subscribe = *subscribe;
        debug!(
            "{}",
            if subscribe {
                "Subscribing"
            } else {
                "Unsubscribing"
            }
        );
        async move {
            let result = async move {
                let push_manager = get_push_manger().await?;

                if subscribe {
                    // Get the servers public key (it's not base64 encoded so no conversion is
                    // needed)
                    let vapid = fetch_vapid().await?;
                    debug!("Got vapid: {:?}", vapid.key);

                    let key = Uint8Array::from(&vapid.key[..]);

                    let mut options = PushSubscriptionOptionsInit::new();
                    options.user_visible_only(true);
                    options.application_server_key(Some(&key));

                    let subscription: PushSubscription =
                        JsFuture::from(push_manager.subscribe_with_options(&options)?)
                            .await?
                            .into();

                    update_subscription(&subscription).await?;
                } else {
                    let subscription: PushSubscription =
                        JsFuture::from(push_manager.get_subscription()?)
                            .await?
                            .into();
                    JsFuture::from(subscription.unsubscribe()?).await?;

                    remove_subscription().await?;
                }

                is_subscribed_resource.refetch();
                Ok::<_, FrontendError<_>>(())
            }
            .await;

            if let Err(e) = result {
                set_error.update(|v| *v = Some(e.to_string()));
            } else {
                set_error.update(|v| *v = None);
            }
        }
    });

    let toggle_subscribe = move || subscribe_action.dispatch(!is_subscribed());

    view! {
        <h1>"Notifications"</h1>
        { move || error.with(|e| e.as_ref().map(|e| view! {
            <p class="error">{e}</p>
        }))}
        { move || if is_subscribed() {
            Some(view! { <p>"You are currently subscribed to receive notifications"</p> })
        } else {
            None
        }}
        <button
            prop:disabled=move || is_subscribed_resource.loading().get() || is_subscribed_resource.get().map(|r| r.is_err()).unwrap_or(true)
            on:click=move |_| toggle_subscribe()
        >
            { move || if is_subscribed() {
                "Unsubscribe"
            } else {
                "Subscribe"
            }}
        </button>
    }
}
