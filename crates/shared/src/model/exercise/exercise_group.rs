use chrono::{DateTime, Utc};

use crate::{feature_model_derives, feature_model_imports, types::Uuid};

feature_model_imports!();

feature_model_derives!(
    "exercise_group",
    "../../../migrations/006-exercise_group/up.sql",
    pub struct ExerciseGroup {
        pub id: Uuid,
        pub name: String,
        pub description: Option<String>,
        pub creation_date: DateTime<Utc>,
        pub last_updated_date: DateTime<Utc>,
    }
);

#[cfg(feature = "wasm")]
impl crate::model::model_into_view::UseDefaultModelView for ExerciseGroup {}

#[cfg(feature = "backend")]
impl ExerciseGroup {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<ExerciseGroup, rusqlite::Error> {
        let (sql, values) = Self::select_star()
            .and_where(Expr::col(ExerciseGroupIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_row(&*values.as_params(), ExerciseGroup::from_row)?;
        Ok(res)
    }
}
