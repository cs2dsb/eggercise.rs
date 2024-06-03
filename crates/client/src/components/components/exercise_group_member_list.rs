use leptos::{component, view, CollectView, ErrorBoundary, IntoView, SignalWith, Transition};
use shared::model::ExerciseGroupMember;

use crate::db::PromiserFetcher;

#[component]
pub fn ExerciseGroupMemberList() -> impl IntoView {
    let exercise_group_members = ExerciseGroupMember::all_resource();

    view! {
        <Transition fallback=move || view! {  <p>"Loading..."</p>} >
            <ErrorBoundary fallback=|errors| view! {
                <div style="color:red">
                    <p>Error loading Exercise Group Member list:</p>
                    <ul>
                    { move || errors.with(|v|
                        v.iter()
                        .map(|(_, e)| view! { <li> { format!("{:?}", e) } </li>})
                        .collect_view())
                    }
                    </ul>
                </div>
            }>
                <h3>ExerciseGroupMember:</h3>
                { move || {
                    exercise_group_members.and_then(|l| l.into_view()).collect_view()
                }}
            </ErrorBoundary>
        </Transition>
    }
}
