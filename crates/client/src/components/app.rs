#[allow(unused_imports)]
use leptos::{
    component, create_memo, create_signal, view, CollectView, For, IntoView, Signal, SignalDispose, SignalUpdate, SignalWith
};
#[allow(unused_imports)]
use leptos_router::{
    Router,
    Route,
    Routes,
    A,
};

#[component]
fn PageA() -> impl IntoView {
    view! {
        <p>"Hello A"</p>
    }
}

#[component]
fn PageB() -> impl IntoView {
    view! {
        <p>"Hello B"</p>
    }
}

#[component]
pub (crate) fn App() -> impl IntoView {
    view! {
        <Router>
            <nav>
                <A href="">"A"</A>
                <span style:margin="10px" />
                <A href="b">"B"</A></nav>
            <Routes>
                <Route path="" view=PageA/>
                <Route path="b" view=PageB/>
            </Routes>
        </Router>
    }
}