use leptos::{component, view, CollectView, IntoView, Transition};
use shared::model::SessionExercise;

use crate::{
    components::FrontendErrorBoundary, db::PromiserFetcher, utils::sqlite3::SqlitePromiserError,
};

#[component]
pub fn SessionExerciseList() -> impl IntoView {
    let session_exercises = SessionExercise::all_resource();

    view! {
        <Transition fallback=move || view! {  <p>"Loading..."</p>} >
            <FrontendErrorBoundary<SqlitePromiserError>>
                <h3>SessionExercise:</h3>
                { move || {
                    session_exercises.and_then(|l| l.into_view()).collect_view()
                }}
            </FrontendErrorBoundary<SqlitePromiserError>>
        </Transition>
    }
}
