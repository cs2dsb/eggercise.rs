use leptos::{component, view, IntoView};

use crate::components::{
    ExerciseGroupList, ExerciseGroupMemberList, ExerciseList, SessionExerciseList, SessionList,
    UserList,
};

#[component]
pub fn Today() -> impl IntoView {
    view! {
        <h2>"Today"</h2>
        <UserList />
        <ExerciseList />
        <ExerciseGroupMemberList />
        <ExerciseGroupList />
        <SessionExerciseList />
        <SessionList />
    }
}
