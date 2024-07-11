use chrono::{DateTime, Utc};

use crate::{feature_model_derives, feature_model_imports, types::Uuid};

feature_model_imports!();

feature_model_derives!(
    "plan",
    "../../../migrations/007-plan/up.sql",
    /// A plan is the a methodology for the exercise programme. Global for all
    /// users
    pub struct Plan {
        pub id: Uuid,
        pub owner_id: Uuid,
        pub name: String,
        pub description: Option<String>,
        pub duration_weeks: u32,
        pub creation_date: DateTime<Utc>,
        pub last_updated_date: DateTime<Utc>,
    }
);

#[cfg(feature = "wasm")]
impl crate::model::model_into_view::UseDefaultModelView for Plan {}

#[cfg(feature = "backend")]
impl Plan {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<Plan, rusqlite::Error> {
        let (sql, values) = Self::select_star()
            .and_where(Expr::col(PlanIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_row(&*values.as_params(), Plan::from_row)?;
        Ok(res)
    }

    pub fn fetch_all(conn: &Connection) -> Result<Vec<Plan>, rusqlite::Error> {
        let (sql, values) = Self::select_star().build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res =
            stmt.query_map(&*values.as_params(), Plan::from_row)?.collect::<Result<_, _>>()?;
        Ok(res)
    }
}
