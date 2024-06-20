use chrono::{DateTime, Utc};

use crate::{feature_model_derives, feature_model_imports, types::Uuid};

feature_model_imports!();

feature_model_derives!(
    "plan_instance",
    "../../../migrations/008-plan_instance/up.sql",
    /// A plan instance is an actual execution of a plan on a given start_date.
    /// Local to a certain user. Can be multiple instances of the same plan
    pub struct PlanInstance {
        pub id: Uuid,
        pub plan_id: Uuid,
        pub user_id: Uuid,
        pub start_date: DateTime<Utc>,
        pub creation_date: DateTime<Utc>,
        pub last_updated_date: DateTime<Utc>,
    }
);

#[cfg(feature = "wasm")]
impl crate::model::model_into_view::UseDefaultModelView for PlanInstance {}

#[cfg(feature = "backend")]
impl PlanInstance {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<PlanInstance, rusqlite::Error> {
        let (sql, values) = Self::select_star()
            .and_where(Expr::col(PlanInstanceIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_row(&*values.as_params(), PlanInstance::from_row)?;
        Ok(res)
    }

    pub fn fetch_all(conn: &Connection) -> Result<Vec<PlanInstance>, rusqlite::Error> {
        let (sql, values) = Self::select_star().build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt
            .query_map(&*values.as_params(), PlanInstance::from_row)?
            .collect::<Result<_, _>>()?;
        Ok(res)
    }
}
