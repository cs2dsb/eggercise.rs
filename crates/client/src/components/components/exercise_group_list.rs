use leptos::{component, view, CollectView, ErrorBoundary, IntoView, SignalWith, Transition};
use shared::model::ExerciseGroup;

use crate::db::PromiserFetcher;

#[component]
pub fn ExerciseGroupList() -> impl IntoView {
    let exercise_groups = ExerciseGroup::all_resource();

    view! {
        <Transition fallback=move || view! {  <p>"Loading..."</p>} >
            <ErrorBoundary fallback=|errors| view! {
                <div style="color:red">
                    <p>Error loading Exercise Group list:</p>
                    <ul>
                    { move || errors.with(|v|
                        v.iter()
                        .map(|(_, e)| view! { <li> { format!("{:?}", e) } </li>})
                        .collect_view())
                    }
                    </ul>
                </div>
            }>
                <h3>ExerciseGroup:</h3>
                { move || {
                    exercise_groups.and_then(|l| l.into_view()).collect_view()
                }}
            </ErrorBoundary>
        </Transition>
    }
}
