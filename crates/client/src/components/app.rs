use leptos::{component, view, IntoView};
use leptos_router::Router;

use crate::{AppNav, AppRoutes};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <small>{ format!("Version: {}{}", 
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_TIME")
                .map(|v| format!("- {v}"))
                .unwrap_or("".to_string())) 
        }</small>
        <Router>
            <AppNav/>
            <AppRoutes/>
        </Router>
    }
}
