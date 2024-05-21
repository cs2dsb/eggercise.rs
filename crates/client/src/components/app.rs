use leptos::{component, view, IntoView};
use leptos_router::Router;

use crate::{AppNav, AppRoutes};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <small>Version: { env!("CARGO_PKG_VERSION") }</small>
        <Router>
            <AppNav/>
            <AppRoutes/>
        </Router>
    }
}
