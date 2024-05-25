use std::time::Duration;

use leptos::{component, create_action, logging::log, set_interval, view, ChildrenFn, IntoView, ReadSignal, SignalUpdate, WriteSignal, Show};

use crate::api::ping;

const ONLINE_CHECK_DELAY: Duration = Duration::from_secs(20);

#[component]
pub fn OnlineCheck(set_online: WriteSignal<bool>) -> impl IntoView {
    let ping_action = create_action(move |_| {
        async move {
            let online = ping().await.is_ok();
            log!("Ping result: {online}");
            set_online.update(|v| *v = online);
        }
    });

    // Do one immediatly 
    ping_action.dispatch(());
    // Then set the interval to do the check periodically
    set_interval(move || {
        ping_action.dispatch(());
    }, ONLINE_CHECK_DELAY);
}

#[component]
pub fn OfflineFallback(
    online: ReadSignal<bool>,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <Show 
            when=online
            fallback=move || view! {
                <h1 style="color:red">"Offline"</h1>
                <p>"The server can't be reached. Try connecting to the internet if you aren't or try again later"</p>
            }
        >
            {children()}
        </Show>
    }
}