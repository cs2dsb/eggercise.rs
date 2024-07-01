use leptos::{
    component, create_effect, create_local_resource, create_signal, create_trigger, view, IntoView,
    SignalGet, SignalUpdate, SignalWith,
};
use shared::api::error::{FrontendError, Nothing};
use tracing::debug;
use wasm_bindgen_futures::JsFuture;

use crate::{
    actions::push_subscription::{subscribe_action, unsubscribe_action},
    utils::browser::get_web_push_manger,
};

#[component]
pub fn Notificiations() -> impl IntoView {
    let subscription_trigger = create_trigger();

    // TODO: This seems to be required because when the trigger (or a signal(())) is
    // used the resource doesn't       refetch. My guess is it's being optimized
    // away
    let (trigger_workaround, set_trigger_workaround) = create_signal(0_usize);

    let is_subscribed_resource = create_local_resource(
        move || trigger_workaround(),
        |_| async move {
            let push_manager = get_web_push_manger().await?;
            let maybe_subscription = JsFuture::from(push_manager.get_subscription()?).await?;
            let is_subscribed =
                !(maybe_subscription.is_null() || maybe_subscription.is_undefined());

            debug!("is_subscribed: {is_subscribed}");
            Ok::<_, FrontendError<Nothing>>(is_subscribed)
        },
    );

    let (error, set_error) = create_signal(None::<String>);

    let is_subscribed = move || {
        subscription_trigger.track();
        is_subscribed_resource
            .get()
            .map(|r| r.unwrap_or(false))
            .unwrap_or(false)
    };

    let subscribe_action = subscribe_action(subscription_trigger, set_error);
    let unsubscribe_action = unsubscribe_action(subscription_trigger, set_error);

    create_effect(move |_| {
        subscription_trigger.track();
        set_trigger_workaround.update(|v| *v += 1);
    });

    let toggle_subscribe = move || {
        if is_subscribed() {
            unsubscribe_action.dispatch(());
        } else {
            subscribe_action.dispatch(());
        }
    };

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
