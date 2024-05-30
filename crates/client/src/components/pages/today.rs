use leptos::{component, view, IntoView};

use crate::components::ExerciseList;

#[component]
pub fn Today() -> impl IntoView {
    view! {
        <p>"Today"</p>
        <ExerciseList />
    }
}
