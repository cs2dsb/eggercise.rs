use leptos::{component, view, CollectView, ErrorBoundary, IntoView, SignalWith, Transition};
use shared::model::Exercise;

use crate::db::model::exercise::get_exercises;

#[component]
pub fn ExerciseListItem<'a>(e: &'a Exercise) -> impl IntoView {
    view! {
        <table>
            <tr><td>Name:</td><td>{ &e.name }</td></tr>
            <tr><td>Id:</td><td>{ format!("{:?}", &e.id) }</td></tr>
            <tr><td>Creation Date:</td><td>{ format!("{:?}", &e.creation_date) }</td></tr>
            <tr><td>Last Updated Date:</td><td>{ format!("{:?}", &e.last_updated_date) }</td></tr>
        </table>
    }
}

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
                <ul>
                { move || {
                    // exercises.and_then(|l| l.iter().map(|e| view! {
                    //     <li>
                    //         <ExerciseListItem e />
                    //     </li>
                    // }).collect_view())
                    exercises.and_then(|l| l.into_view()).collect_view()
                }}
                </ul>
            </ErrorBoundary>
        </Transition>
    }
}
