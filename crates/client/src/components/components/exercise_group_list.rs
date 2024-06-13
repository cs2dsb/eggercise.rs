use leptos::{component, view, CollectView, IntoView, Transition};
use shared::model::ExerciseGroup;

use crate::{
    components::FrontendErrorBoundary, db::PromiserFetcher, utils::sqlite3::SqlitePromiserError,
};

#[component]
pub fn ExerciseGroupList() -> impl IntoView {
    let exercise_groups = ExerciseGroup::all_resource();

    view! {
        <Transition fallback=move || view! {  <p>"Loading..."</p>} >
            <FrontendErrorBoundary<SqlitePromiserError>>
                <h3>ExerciseGroup:</h3>
                { move || {
                    exercise_groups.and_then(|l| l.into_view()).collect_view()
                }}
            </FrontendErrorBoundary<SqlitePromiserError>>
        </Transition>
    }
}
