use leptos::{component, use_context, view, IntoView, Resource};

use crate::db::migrations::{DatabaseVersion, MigrationError};

#[component]
pub fn Debug() -> impl IntoView {
    let db_version: Resource<(), Result<DatabaseVersion, MigrationError>> = use_context()
        .expect("Failed to find DatabaseVersion resource in context"); 

    view! {
        <h1>"Debug"</h1>
        <p>"Database Version: " { move || db_version
            .and_then(|v| v.into_view())
        }</p>
    }
}
