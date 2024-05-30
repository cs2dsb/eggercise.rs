use std::time::Duration;

use leptos::{component, create_local_resource_with_initial_value, 
    provide_context, SignalGet, set_interval, use_context, view, ChildrenFn, IntoView, Resource, Show};
use tracing::{ debug, error };

use crate::api::ping;

const ONLINE_CHECK_DELAY: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy)]
pub struct Online (Resource<(), bool>);

impl Online {
    pub fn get(&self) -> bool {
        self.0.get().unwrap_or(false)
    }

    pub fn provide_context() {
        let online = create_local_resource_with_initial_value(|| (), move |_|
            async move {
                let online = ping().await.is_ok();
                debug!("Ping result: {online}");
                online
            }, Some(false));

        provide_context(Online(online));

        // Set the interval to do the check periodically
        set_interval(|| {
            if let Some(online) = use_context::<Online>() {
                online.0.refetch();
            } else {
                error!("Online resource missing from context!");
            }
        }, ONLINE_CHECK_DELAY);

    }

    pub fn use_online() -> Self {
        use_context()
            .expect("Online resource missing from context!")
    }
}

#[component]
pub fn OnlineCheck() -> impl IntoView {
    let online = Online::use_online();
    view! {
        { 
            move || if online.get() {
                "🟢 Online"
            } else {
                "🔴 Offline"
            }
        }
    }
}

#[component]
pub fn OfflineFallback(
    children: ChildrenFn,
) -> impl IntoView {
    let online = Online::use_online();

    view! {
        <Show 
            when=move || online.get()
            fallback=move || view! {
                <h1 style="color:red">"Offline"</h1>
                <p>"The server can't be reached. Try connecting to the internet if you aren't or try again later"</p>
            }
        >
            {children()}
        </Show>
    }
}