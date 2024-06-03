use leptos::{component, view, IntoView};

use crate::components::{
    ExerciseGroupList, ExerciseGroupMemberList, ExerciseList, SessionExerciseList, SessionList,
};

#[component]
pub fn Today() -> impl IntoView {
    view! {
        <h2>"Today"</h2>
        <ExerciseList />
        <ExerciseGroupMemberList />
        <ExerciseGroupList />
        <SessionExerciseList />
        <SessionList />
    }
}
