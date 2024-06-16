use leptos::{component, view, CollectView, IntoView, Transition};
use shared::model::User;

use crate::{
    components::FrontendErrorBoundary, db::PromiserFetcher, utils::sqlite3::SqlitePromiserError,
};

#[component]
pub fn UserList() -> impl IntoView {
    let user = User::all_resource();

    view! {
        <Transition fallback=move || view! {  <p>"Loading..."</p>} >
            <FrontendErrorBoundary<SqlitePromiserError>>
                <h3>User:</h3>
                { move || {
                    user.and_then(|l| l.into_view()).collect_view()
                }}
            </FrontendErrorBoundary<SqlitePromiserError>>
        </Transition>
    }
}
