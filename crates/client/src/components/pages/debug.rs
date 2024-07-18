use leptos::{component, use_context, view, For, IntoView, Resource, SignalWith};

// use shared::model::{
//     Exercise, ExerciseGroup, ExerciseGroupMember, Plan, PlanExerciseGroup,
// PlanInstance, Session,     SessionExercise, User,
// };
use crate::{
    // components::ModelList,
    db::migrations::{DatabaseVersion, MigrationError},
    utils::websocket::Websocket,
};

#[component]
pub fn Debug() -> impl IntoView {
    let db_version: Resource<(), Result<DatabaseVersion, MigrationError>> =
        use_context().expect("Failed to find DatabaseVersion resource in context");

    let ws = Websocket::use_websocket();
    let status = ws.status_signal();
    let message = ws.message_signal();

    view! {
        <h1>"Debug"</h1>
        <p>"Database Version: " { move || db_version
            .and_then(|v| v.into_view())
        }</p>
        <p>"Websocket status: " { move ||  status.with(|v| view! { {format!("{v:?}")} }) }</p>
        <p>"Websocket messages: " { move ||  view! {
            <For
                each=move || message.with(|v| v.iter().enumerate().map(|(i, v)| (i, format!("{:?}", v))).collect::<Vec<_>>())
                key=|v| v.0
                children=move |v| view! { <p> { v.1 } </p> }
            />
        }}</p>
    }
    // <ModelList<User> />
    // <ModelList<Exercise> />
    // <ModelList<ExerciseGroup> />
    // <ModelList<ExerciseGroupMember> />
    // <ModelList<Plan> />
    // <ModelList<PlanExerciseGroup> />
    // <ModelList<PlanInstance> />
    // <ModelList<Session> />
    // <ModelList<SessionExercise> />
}
