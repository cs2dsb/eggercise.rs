use leptos::{component, view, CollectView, ErrorBoundary, IntoView, SignalWith, Transition};
use shared::model::SessionExercise;

use crate::db::PromiserFetcher;

#[component]
pub fn SessionExerciseList() -> impl IntoView {
    let session_exercises = SessionExercise::all_resource();

    view! {
        <Transition fallback=move || view! {  <p>"Loading..."</p>} >
            <ErrorBoundary fallback=|errors| view! {
                <div style="color:red">
                    <p>Error loading Session Exercise list:</p>
                    <ul>
                    { move || errors.with(|v|
                        v.iter()
                        .map(|(_, e)| view! { <li> { format!("{:?}", e) } </li>})
                        .collect_view())
                    }
                    </ul>
                </div>
            }>
                <h3>SessionExercise:</h3>
                { move || {
                    session_exercises.and_then(|l| l.into_view()).collect_view()
                }}
            </ErrorBoundary>
        </Transition>
    }
}
