use chrono::Utc;
use futures::{future::join_all, TryFutureExt};
use leptos::{
    component, create_action, create_local_resource, view, CollectView, IntoView, Resource,
    Transition,
};
use shared::{
    model::{
        Exercise, ExerciseGroup, ExerciseGroupIden, ExerciseGroupMember, ExerciseGroupMemberIden,
        ExerciseIden, Plan, PlanExerciseGroup, PlanExerciseGroupIden, PlanIden, PlanInstance,
        PlanInstanceIden, Session, SessionExercise, SessionExerciseIden, SessionIden, User,
        UserExercise, UserExerciseIden,
    },
    types::Uuid,
};
use tracing::debug;
use web_time::Instant;

use crate::{
    components::FrontendErrorBoundary,
    db::{PromiserFetcher, PromiserInserter},
    utils::sqlite3::{SqlitePromiser, SqlitePromiserError},
};

fn plan() -> Resource<
    (),
    Result<
        Vec<(
            Plan,
            PlanInstance,
            Vec<(
                PlanExerciseGroup,
                ExerciseGroup,
                Vec<(
                    Exercise,
                    Option<UserExercise>,
                    Vec<(SessionExercise, Session)>,
                )>,
            )>,
        )>,
        SqlitePromiserError,
    >,
> {
    create_local_resource(
        || (),
        |_| async {
            let start = Instant::now();
            let user = {
                let mut users = <User as PromiserFetcher>::fetch_all().await?;
                if users.len() != 1 {
                    Err(SqlitePromiserError::ExecResult(format!(
                        "Expected 1 user but got {}",
                        users.len()
                    )))?;
                }
                users.pop().unwrap()
            };
            debug!("User: {:?}", user);

            let plan_instances = PlanInstance::fetch_by(&user.id, PlanInstanceIden::UserId).await?;
            debug!("Plan instances: {:?}", plan_instances);

            let plans = join_all(
                plan_instances
                    .iter()
                    .map(|pi| Plan::fetch_one_by(&pi.plan_id, PlanIden::Id)),
            )
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
            debug!("Plans: {:?}", plans);

            let plan_exercise_groups =
                join_all(plan_instances.iter().map(|pi| {
                    PlanExerciseGroup::fetch_by(&pi.plan_id, PlanExerciseGroupIden::PlanId)
                }))
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            debug!("Plan exercise groups: {:?}", plan_exercise_groups);

            let exercise_groups = join_all(plan_exercise_groups.iter().map(|pigs| {
                join_all(pigs.iter().map(|pig| {
                    ExerciseGroup::fetch_one_by(&pig.exercise_group_id, ExerciseGroupIden::Id)
                }))
            }))
            .await
            .into_iter()
            .map(|r| r.into_iter().collect::<Result<Vec<_>, _>>())
            .collect::<Result<Vec<_>, _>>()?;
            debug!("Exercise groups: {:?}", exercise_groups);

            let exercises = join_all(exercise_groups.iter().map(|egs| {
                join_all(egs.iter().map(|eg| {
                    ExerciseGroupMember::fetch_by(&eg.id, ExerciseGroupMemberIden::GroupId)
                        .and_then(|egms| async move {
                            join_all(egms.into_iter().map(|egm| {
                                Exercise::fetch_one_by(egm.exercise_id, ExerciseIden::Id)
                            }))
                            .await
                            .into_iter()
                            .collect::<Result<Vec<_>, _>>()
                        })
                }))
            }))
            .await
            .into_iter()
            .map(|r| r.into_iter().collect::<Result<Vec<_>, _>>())
            .collect::<Result<Vec<_>, _>>()?;
            debug!("Exercises: {:?}", exercises);

            let user_exercises = join_all(exercises.iter().map(|es| {
                join_all(es.iter().map(|es| async move {
                    join_all(es.iter().map(|e| {
                        UserExercise::fetch_maybe_one_by(&e.id, UserExerciseIden::ExerciseId)
                    }))
                    .await
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
                }))
            }))
            .await
            .into_iter()
            .map(|r| r.into_iter().collect::<Result<Vec<_>, _>>())
            .collect::<Result<Vec<_>, _>>()?;
            debug!("User exercises: {:?}", user_exercises);

            let exercises_sessions = join_all(exercises.iter().map(|exs| {
                join_all(exs.iter().map(|exs| async move {
                    join_all(exs.iter().map(|e| {
                        SessionExercise::fetch_by(&e.id, SessionExerciseIden::ExerciseId).and_then(
                            |ses| async move {
                                join_all(ses.into_iter().map(|se| async move {
                                    let session =
                                        Session::fetch_one_by(&se.session_id, SessionIden::Id)
                                            .await?;
                                    Ok((se, session))
                                }))
                                .await
                                .into_iter()
                                .collect::<Result<Vec<_>, _>>()
                            },
                        )
                    }))
                    .await
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
                }))
            }))
            .await
            .into_iter()
            .map(|r| r.into_iter().collect::<Result<Vec<_>, _>>())
            .collect::<Result<Vec<_>, _>>()?;
            debug!("Exercise sessions: {:?}", exercises_sessions);

            let ret = plans
                .into_iter()
                .zip(plan_instances.into_iter())
                .zip(plan_exercise_groups.into_iter())
                .zip(exercise_groups.into_iter())
                .zip(exercises.into_iter())
                .zip(user_exercises.into_iter())
                .zip(exercises_sessions.into_iter())
                .map(
                    |(
                        (
                            (
                                (((plan, plan_instance), plan_exercise_groups), exercise_groups),
                                exercises,
                            ),
                            user_exercises,
                        ),
                        exercises_sessions,
                    )| {
                        (
                            plan,
                            plan_instance,
                            plan_exercise_groups
                                .into_iter()
                                .zip(exercise_groups.into_iter())
                                .zip(exercises.into_iter())
                                .zip(user_exercises.into_iter())
                                .zip(exercises_sessions.into_iter())
                                .map(
                                    |(
                                        (
                                            ((plan_exercise_group, exercise_group), exercises),
                                            user_exercises,
                                        ),
                                        exercises_sessions,
                                    )| {
                                        (
                                        plan_exercise_group,
                                        exercise_group,
                                        exercises
                                            .into_iter()
                                            .zip(user_exercises.into_iter())
                                            .zip(exercises_sessions)
                                            .map(|((exercise, user_exercise), exercise_sessions)| (
                                                exercise,
                                                user_exercise,
                                                exercise_sessions,
                                            ))
                                            .collect::<Vec<_>>(),
                                    )
                                    },
                                )
                                .collect::<Vec<_>>(),
                        )
                    },
                )
                .collect::<Vec<_>>();

            debug!("today resource took: {:.2}", start.elapsed().as_secs_f32());

            Ok(ret)
        },
    )
}

#[component]
pub fn Today() -> impl IntoView {
    let plans = plan();

    view! {
        <Transition fallback=move || view! {  <p>"Loading..."</p>} >
            <FrontendErrorBoundary<SqlitePromiserError>>
                <h2>"Today"</h2>
                { move || {
                    plans.and_then(|p| p
                        .into_iter()
                        .map(|(plan, plan_instance, groups)| view ! {
                            <Plan plan plan_instance groups />
                        })
                        .collect_view())
                    .collect_view()
                }}
            </FrontendErrorBoundary<SqlitePromiserError>>
        </Transition>
    }
}

#[component]
fn Plan<'a>(
    plan: &'a Plan,
    plan_instance: &'a PlanInstance,
    groups: &'a Vec<(
        PlanExerciseGroup,
        ExerciseGroup,
        Vec<(
            Exercise,
            Option<UserExercise>,
            Vec<(SessionExercise, Session)>,
        )>,
    )>,
) -> impl IntoView {
    view! {
        <div>
            <h3>{ &plan.name }</h3>
            { plan.description.as_ref().map(|d| view! { <p>Description: { d }</p> }) }
            <p>{ format!("Start date: {}", plan_instance.start_date) }</p>
            { groups.into_iter().map(|(plan_group, group, exercises)| view! {
                <PlanGroup plan_instance plan_group group exercises />
            }).collect_view() }
        </div>
    }
}

#[component]
fn PlanGroup<'a>(
    plan_instance: &'a PlanInstance,
    plan_group: &'a PlanExerciseGroup,
    group: &'a ExerciseGroup,
    exercises: &'a Vec<(
        Exercise,
        Option<UserExercise>,
        Vec<(SessionExercise, Session)>,
    )>,
) -> impl IntoView {
    view! {
        <div>
            <h4>{ &group.name }</h4>
            { group.description.as_ref().map(|d| view! { <p>Description: { d }</p> }) }
            { plan_group.notes.as_ref().map(|n| view! { <p>Notes: { n }</p> }) }
            <div>
                { exercises.into_iter().map(|(exercise, user_exercise, exercise_sessions)| view ! {
                    <Exercise plan_instance_id=plan_instance.id exercise user_exercise exercise_sessions/>
                }).collect_view() }
            </div>
        </div>
    }
}

#[component]
fn Exercise<'a>(
    plan_instance_id: Uuid,
    exercise: &'a Exercise,
    user_exercise: &'a Option<UserExercise>,
    exercise_sessions: &'a Vec<(SessionExercise, Session)>,
) -> impl IntoView {
    let now = Utc::now();
    let _most_recent_session = exercise_sessions
        .iter()
        .filter(|(_, session)| session.planned_date < now)
        .max_by(|(_, session_a), (_, session_b)| {
            session_a.planned_date.cmp(&session_b.planned_date)
        });

    let create_new_session_action =
        create_action(move |(exercise, plan_instance): &(Uuid, Uuid)| {
            let promiser = SqlitePromiser::use_promiser();
            let now = Utc::now();

            let session = Session {
                id: Uuid::new_v4(),
                plan_instance_id: *plan_instance,
                planned_date: now,
                performed_date: None,
                creation_date: now,
                last_updated_date: now,
            };

            let session_exercise = SessionExercise {
                id: Uuid::new_v4(),
                exercise_id: *exercise,
                session_id: session.id,
                planned_sets: Default::default(),
                performed_sets: Default::default(),
                creation_date: now,
                last_updated_date: now,
            };

            async move {
                promiser.exec(session.insert_sql()?).await?;
                promiser.exec(session_exercise.insert_sql()?).await?;

                Ok::<_, SqlitePromiserError>(())
            }
        });

    let exercise_id = exercise.id;

    view! {
        <div>
            <h5>{ &exercise.name }</h5>
            <p>"Recovery days: " { format!("{:.1}",
                user_exercise.as_ref()
                    .map(|ue| ue.recovery_days)
                    .flatten()
                    .unwrap_or(exercise.base_recovery_days))
            }</p>
            { exercise.description.as_ref().map(|d| view! { <p>Description: { d }</p> }) }
            {if exercise_sessions.len() > 0 {
                exercise_sessions.into_iter().map(|(session_exercise, session)| view! {
                    <ExerciseSession session_exercise session />
                }).collect_view()
            } else {
                view! {
                    <form on:submit=|ev| ev.prevent_default()>
                        <button
                            on:click=move |_| create_new_session_action.dispatch((exercise_id, plan_instance_id))
                        >
                            "Create session"
                        </button>
                    </form>
                }.into_view()
            }}
        </div>
    }
}

#[component]
fn ExerciseSession<'a>(
    #[allow(unused_variables)] session_exercise: &'a SessionExercise,
    session: &'a Session,
) -> impl IntoView {
    view! {
        <div>
            <h6>"Session: " { format!("{}", session.planned_date) }</h6>
        </div>
    }
}
