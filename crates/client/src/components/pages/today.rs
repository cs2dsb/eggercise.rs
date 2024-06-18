use futures::{future::join_all, TryFutureExt};
use leptos::{component, create_local_resource, view, CollectView, IntoView, Resource, Transition};
use shared::model::{
    Exercise, ExerciseGroup, ExerciseGroupIden, ExerciseGroupMember, ExerciseGroupMemberIden,
    ExerciseIden, Plan, PlanExerciseGroup, PlanExerciseGroupIden, PlanIden, PlanInstance,
    PlanInstanceIden, User,
};
use tracing::debug;
use web_time::Instant;

use crate::{
    components::FrontendErrorBoundary, db::PromiserFetcher, utils::sqlite3::SqlitePromiserError,
};

fn plan() -> Resource<
    (),
    Result<
        Vec<(
            Plan,
            PlanInstance,
            Vec<(PlanExerciseGroup, ExerciseGroup, Vec<Exercise>)>,
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
                            .map(|v| v)
                            .collect::<Result<Vec<_>, _>>()
                        })
                }))
            }))
            .await
            .into_iter()
            .map(|r| r.into_iter().collect::<Result<Vec<_>, _>>())
            .collect::<Result<Vec<_>, _>>()?;
            debug!("Exercises: {:?}", exercises);

            let ret = plans
                .into_iter()
                .zip(plan_instances.into_iter())
                .zip(plan_exercise_groups.into_iter())
                .zip(exercise_groups.into_iter())
                .zip(exercises.into_iter())
                .map(
                    |(
                        (((plan, plan_instance), plan_exercise_groups), exercise_groups),
                        exercises,
                    )| {
                        (
                            plan,
                            plan_instance,
                            plan_exercise_groups
                                .into_iter()
                                .zip(exercise_groups.into_iter())
                                .zip(exercises.into_iter())
                                .map(|((plan_exercise_group, exercise_group), exercises)| {
                                    (plan_exercise_group, exercise_group, exercises)
                                })
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
                            <div>
                                <h3>{ &plan.name }</h3>
                                { plan.description.as_ref().map(|d| view! { <p>Description: { d }</p> }) }
                                <p>{ format!("Start date: {}", plan_instance.start_date) }</p>
                                { groups.into_iter().map(|(plan_group, group, exercises)| view! {
                                    <div>
                                        <h4>{ &group.name }</h4>
                                        { group.description.as_ref().map(|d| view! { <p>Description: { d }</p> }) }
                                        { plan_group.notes.as_ref().map(|n| view! { <p>Notes: { n }</p> }) }
                                        <div>
                                            { exercises.into_iter().map(|exercise| view ! {
                                                <div>
                                                    <h5>{ &exercise.name }</h5>
                                                    { exercise.description.as_ref().map(|d| view! { <p>Description: { d }</p> }) }
                                                </div>
                                            }).collect_view() }
                                        </div>
                                    </div>
                                }).collect_view() }
                            </div>
                        })
                        .collect_view())
                    .collect_view()
                }}
            </FrontendErrorBoundary<SqlitePromiserError>>
        </Transition>
    }
}
