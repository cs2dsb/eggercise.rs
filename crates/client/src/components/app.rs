#[allow(unused_imports)]
use leptos::{
    component, create_memo, create_signal, view, CollectView, For, IntoView, Signal, SignalDispose,
    SignalUpdate, SignalWith,
};
#[allow(unused_imports)]
use leptos_router::{Route, Router, Routes, A};

use super::{AppNav, AppRoutes};

#[component]
pub(crate) fn App() -> impl IntoView {
    view! {
        <small>Version: { env!("CARGO_PKG_VERSION") }</small>
        <Router>
            <AppNav/>
            <AppRoutes/>
        </Router>
    }
}
