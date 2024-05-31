use chrono::{DateTime, Utc};

use crate::{ 
    types::Uuid,
    feature_model_imports,
    feature_model_derives,
};

feature_model_imports!();

feature_model_derives!(
    "exercise", 
    "../../../migrations/004-exercise/up.sql",
    pub struct Exercise {
        pub id:                  Uuid,
        pub name:                String,
        pub description:         Option<String>,
        pub creation_date:       DateTime<Utc>,
        pub last_updated_date:   DateTime<Utc>,
    }
);

#[cfg(feature = "sea-query-enum")]
const EXERCISE_STAR: [ExerciseIden; 5]= [
    ExerciseIden::Id,
    ExerciseIden::Name,
    ExerciseIden::Description,
    ExerciseIden::CreationDate,
    ExerciseIden::LastUpdatedDate,
];

#[cfg(feature = "backend")]
impl Exercise {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<Exercise, rusqlite::Error> {
        let (sql, values) = Query::select()
            .columns(EXERCISE_STAR)
            .from(ExerciseIden::Table)
            .and_where(Expr::col(ExerciseIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_row(&*values.as_params(), Exercise::from_row)?;
        Ok(res)
    }

    pub fn fetch_all(conn: &Connection) -> Result<Vec<Exercise>, rusqlite::Error> {
        let (sql, values) = Query::select()
            .columns(EXERCISE_STAR)
            .from(ExerciseIden::Table)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_map(&*values.as_params(), Exercise::from_row)?
            .collect::<Result<_, _>>()?;
        Ok(res)
    }
}

#[cfg(feature = "frontend")]
impl Exercise {
    pub fn fetch_all_get_sql() -> String {
        Query::select()
            .columns(EXERCISE_STAR)
            .from(ExerciseIden::Table)
            .to_string(SqliteQueryBuilder)
    }
}