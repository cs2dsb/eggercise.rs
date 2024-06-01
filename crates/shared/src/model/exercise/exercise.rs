use chrono::{DateTime, Utc};

use crate::{feature_model_derives, feature_model_imports, types::Uuid};

feature_model_imports!();

feature_model_derives!(
    "exercise",
    "../../../migrations/004-exercise/up.sql",
    pub struct Exercise {
        pub id: Uuid,
        pub name: String,
        pub description: Option<String>,
        pub creation_date: DateTime<Utc>,
        pub last_updated_date: DateTime<Utc>,
    }
);

#[cfg(feature = "backend")]
impl Exercise {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<Exercise, rusqlite::Error> {
        let (sql, values) = Self::select_star()
            .and_where(Expr::col(ExerciseIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_row(&*values.as_params(), Exercise::from_row)?;
        Ok(res)
    }

    pub fn fetch_all(conn: &Connection) -> Result<Vec<Exercise>, rusqlite::Error> {
        let (sql, values) = Self::select_star().build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt
            .query_map(&*values.as_params(), Exercise::from_row)?
            .collect::<Result<_, _>>()?;
        Ok(res)
    }
}
