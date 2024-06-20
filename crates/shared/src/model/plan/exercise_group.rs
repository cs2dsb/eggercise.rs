use chrono::{DateTime, Utc};

use super::PlanConfig;
use crate::{feature_model_derives, feature_model_imports, types::Uuid};

feature_model_imports!();

feature_model_derives!(
    "plan_exercise_group",
    "../../../migrations/009-plan_exercise_group/up.sql",
    /// A plan PlanExerciseGroup group is a sub group of PlanExerciseGroups that
    /// are configured on a given plan. This is the level at which
    /// progression is programmed. Group can contain one or many
    /// PlanExerciseGroups. Plan can contain one or more groups
    pub struct PlanExerciseGroup {
        pub id: Uuid,
        pub plan_id: Uuid,
        pub exercise_group_id: Uuid,
        pub notes: Option<String>,
        pub config: Option<PlanConfig>,
        pub creation_date: DateTime<Utc>,
        pub last_updated_date: DateTime<Utc>,
    }
);

#[cfg(feature = "wasm")]
impl crate::model::model_into_view::UseDefaultModelView for PlanExerciseGroup {}

#[cfg(feature = "backend")]
impl PlanExerciseGroup {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<PlanExerciseGroup, rusqlite::Error> {
        let (sql, values) = Self::select_star()
            .and_where(Expr::col(PlanExerciseGroupIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_row(&*values.as_params(), PlanExerciseGroup::from_row)?;
        Ok(res)
    }

    pub fn fetch_all(conn: &Connection) -> Result<Vec<PlanExerciseGroup>, rusqlite::Error> {
        let (sql, values) = Self::select_star().build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt
            .query_map(&*values.as_params(), PlanExerciseGroup::from_row)?
            .collect::<Result<_, _>>()?;
        Ok(res)
    }
}
