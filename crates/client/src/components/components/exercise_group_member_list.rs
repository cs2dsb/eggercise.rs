use leptos::{component, view, CollectView, IntoView, Transition};
use shared::model::ExerciseGroupMember;

use crate::{
    components::FrontendErrorBoundary, db::PromiserFetcher, utils::sqlite3::SqlitePromiserError,
};

#[component]
pub fn ExerciseGroupMemberList() -> impl IntoView {
    let exercise_group_members = ExerciseGroupMember::all_resource();

    view! {
        <Transition fallback=move || view! {  <p>"Loading..."</p>} >
            <FrontendErrorBoundary<SqlitePromiserError>>
                <h3>ExerciseGroupMember:</h3>
                { move || {
                    exercise_group_members.and_then(|l| l.into_view()).collect_view()
                }}
            </FrontendErrorBoundary<SqlitePromiserError>>
        </Transition>
    }
}
