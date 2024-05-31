use chrono::{DateTime, Utc};

use crate::{feature_model_derives, feature_model_imports, types::Uuid};

feature_model_imports!();

feature_model_derives!(
    "session_exercise",
    "../../../migrations/006-session_exercise/up.sql",
    pub struct SessionExercise {
        pub id: Uuid,
        pub exercise_id: Uuid,
        pub session_id: Uuid,
        pub creation_date: DateTime<Utc>,
        pub last_updated_date: DateTime<Utc>,
    }
);

#[cfg(feature = "sea-query-enum")]
const SESSION_EXERCISE_STAR: [SessionExerciseIden; 5] = [
    SessionExerciseIden::Id,
    SessionExerciseIden::ExerciseId,
    SessionExerciseIden::SessionId,
    SessionExerciseIden::CreationDate,
    SessionExerciseIden::LastUpdatedDate,
];

#[cfg(feature = "backend")]
impl SessionExercise {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<SessionExercise, rusqlite::Error> {
        let (sql, values) = Query::select()
            .columns(SESSION_EXERCISE_STAR)
            .from(SessionExerciseIden::Table)
            .and_where(Expr::col(SessionExerciseIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_row(&*values.as_params(), SessionExercise::from_row)?;
        Ok(res)
    }
}
