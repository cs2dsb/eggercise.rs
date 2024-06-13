use leptos::{component, view, CollectView, IntoView, Transition};
use shared::model::Session;

use crate::{
    components::FrontendErrorBoundary, db::PromiserFetcher, utils::sqlite3::SqlitePromiserError,
};

#[component]
pub fn SessionList() -> impl IntoView {
    let sessions = Session::all_resource();

    view! {
        <Transition fallback=move || view! {  <p>"Loading..."</p>} >
            <FrontendErrorBoundary<SqlitePromiserError>>
                <h3>SessionList:</h3>
                { move || {
                    sessions.and_then(|l| l.into_view()).collect_view()
                }}
            </FrontendErrorBoundary<SqlitePromiserError>>
        </Transition>
    }
}
