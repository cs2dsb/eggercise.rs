use leptos::{component, view, CollectView, ErrorBoundary, IntoView, SignalWith, Transition};
use shared::model::Exercise;

use crate::db::PromiserFetcher;

#[component]
pub fn ExerciseList() -> impl IntoView {
    let exercises = Exercise::all_resource();

    view! {
        <Transition fallback=move || view! {  <p>"Loading..."</p>} >
            <ErrorBoundary fallback=|errors| view! {
                <div style="color:red">
                    <p>Error loading exercise list:</p>
                    <ul>
                    { move || errors.with(|v|
                        v.iter()
                        .map(|(_, e)| view! { <li> { format!("{:?}", e) } </li>})
                        .collect_view())
                    }
                    </ul>
                </div>
            }>
                <h3>Exercise:</h3>
                { move || {
                    exercises.and_then(|l| l.into_view()).collect_view()
                }}
            </ErrorBoundary>
        </Transition>
    }
}
