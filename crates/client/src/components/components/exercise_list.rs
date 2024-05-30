use leptos::{component, view, CollectView, ErrorBoundary, IntoView, SignalWith, Transition};

use crate::db::model::exercise::get_exercises;

#[component]
pub fn ExerciseList() -> impl IntoView {
    let exercises = get_exercises();

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
                // <p><small>"Opfs tree: "{ dbsetup.and_then(|v| format!("{:#?}", v.0)) }</small></p>
                <ul>
                { move || {
                    exercises.and_then(|l| l.iter().map(|e| view! {
                        <li>
                            <span>{ &e.name }</span>
                            <span>{ &e.id }</span>
                            <span>{ &e.creation_date }</span>
                            <span>{ &e.last_updated_date }</span>
                        </li>
                    }).collect_view())
                }}
                </ul>
            </ErrorBoundary>
        </Transition>
    }
}
