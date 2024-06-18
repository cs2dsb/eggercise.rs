use std::collections::HashMap;

use chrono::{DateTime, Utc};
#[cfg(feature = "backend")]
use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
    ToSql,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::{
    Exercise, ExerciseGroup, Plan, PlanExerciseGroup, PlanInstance, Session, SessionExercise,
};

pub enum PlanOutcome {
    CreateSession(Session, Vec<SessionExercise>),
}

pub trait Planner {
    fn plan(
        // TODO: can probably slim these down once some examples have been implemented
        plan: &Plan,
        plan_instance: &PlanInstance,
        plan_exercise_group: &PlanExerciseGroup,
        exercise_group: &ExerciseGroup,
        exercises: &HashMap<Uuid, Exercise>,
        shared_config: &SharedConfig,
        current_date: DateTime<Utc>,
    ) -> Vec<PlanOutcome>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WeeklyUndulatingConfig {}

impl Planner for WeeklyUndulatingConfig {
    #[allow(unused_variables)]
    fn plan(
        // TODO: can probably slim these down once some examples have been implemented
        plan: &Plan,
        plan_instance: &PlanInstance,
        plan_exercise_group: &PlanExerciseGroup,
        exercise_group: &ExerciseGroup,
        exercises: &HashMap<Uuid, Exercise>,

        shared_config: &SharedConfig,
        current_date: DateTime<Utc>,
    ) -> Vec<PlanOutcome> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlanAlgorithm {
    WeeklyUndulating(WeeklyUndulatingConfig),
}

impl Default for PlanAlgorithm {
    fn default() -> Self {
        Self::WeeklyUndulating(Default::default())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SharedConfig {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PlanConfig {
    pub algorithm: PlanAlgorithm,
    pub shared_config: SharedConfig,
}

#[cfg(feature = "backend")]
impl ToSql for PlanConfig {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        serde_json::to_string_pretty(self)
            .map(ToSqlOutput::from)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    }
}

#[cfg(feature = "backend")]
impl FromSql for PlanConfig {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        <serde_json::Value as FromSql>::column_result(value)
            .and_then(|v| serde_json::from_value(v).map_err(|e| FromSqlError::Other(Box::new(e))))
    }
}
