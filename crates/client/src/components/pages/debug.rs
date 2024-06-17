use leptos::{component, use_context, view, IntoView, Resource};
use shared::model::{
    Exercise, ExerciseGroup, ExerciseGroupMember, Plan, PlanExerciseGroup, PlanInstance, Session,
    SessionExercise, User,
};

use crate::{
    components::ModelList,
    db::migrations::{DatabaseVersion, MigrationError},
};

#[component]
pub fn Debug() -> impl IntoView {
    let db_version: Resource<(), Result<DatabaseVersion, MigrationError>> =
        use_context().expect("Failed to find DatabaseVersion resource in context");

    view! {
        <h1>"Debug"</h1>
        <p>"Database Version: " { move || db_version
            .and_then(|v| v.into_view())
        }</p>

        <ModelList<User> />
        <ModelList<Exercise> />
        <ModelList<ExerciseGroup> />
        <ModelList<ExerciseGroupMember> />
        <ModelList<Plan> />
        <ModelList<PlanExerciseGroup> />
        <ModelList<PlanInstance> />
        <ModelList<Session> />
        <ModelList<SessionExercise> />
    }
}
