use leptos::{component, view, CollectView, ErrorBoundary, IntoView, SignalWith, Transition};
use shared::model::Session;

use crate::db::PromiserFetcher;

#[component]
pub fn SessionList() -> impl IntoView {
    let sessions = Session::all_resource();

    view! {
        <Transition fallback=move || view! {  <p>"Loading..."</p>} >
            <ErrorBoundary fallback=|errors| view! {
                <div style="color:red">
                    <p>Error loading Session list:</p>
                    <ul>
                    { move || errors.with(|v|
                        v.iter()
                        .map(|(_, e)| view! { <li> { format!("{:?}", e) } </li>})
                        .collect_view())
                    }
                    </ul>
                </div>
            }>
                <h3>SessionList:</h3>
                { move || {
                    sessions.and_then(|l| l.into_view()).collect_view()
                }}
            </ErrorBoundary>
        </Transition>
    }
}
