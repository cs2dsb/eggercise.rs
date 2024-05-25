use leptos::{component, create_signal, view, IntoView, SignalGet};
use leptos_router::Router;

use crate::{AppNav, AppRoutes, components::OnlineCheck};

#[component]
pub fn App() -> impl IntoView {
    let (online, set_online) = create_signal(false);
    view! {
        <OnlineCheck set_online/>
        <div>
            <small>{ format!("Version: {}{}", 
                env!("CARGO_PKG_VERSION"),
                option_env!("BUILD_TIME")
                    .map(|v| format!(" - {v}"))
                    .unwrap_or("".to_string())) 
            }</small>
        </div>
        <div>
        <small>"Online: "{ move || online.get().to_string() }</small>
        </div>
        <Router>
            <AppNav/>
            <AppRoutes online/>
        </Router>
    }
}
