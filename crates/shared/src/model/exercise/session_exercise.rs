use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "backend")]
use {
    exemplar::Model,
    rusqlite::Connection,
    sea_query::{enum_def, Expr, Query, SqliteQueryBuilder},
    sea_query_rusqlite::RusqliteBinder,
};

use crate::types::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "backend", derive(Model))]
#[cfg_attr(feature = "backend", table("session_exercise"))]
#[cfg_attr(
    feature = "backend",
    check("../../../migrations/006-session_exercise/up.sql")
)]
#[cfg_attr(feature = "backend", enum_def)]
pub struct SessionExercise {
    pub id:                  Uuid,
    pub exercise_id:         Uuid,
    pub session_id:          Uuid,
    pub creation_date:       DateTime<Utc>,
    pub last_updated_date:   DateTime<Utc>,
}

#[cfg(feature = "backend")]
impl SessionExercise {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<SessionExercise, rusqlite::Error> {
        let (sql, values) = Query::select()
            .columns([
                SessionExerciseIden::Id,
                SessionExerciseIden::ExerciseId,
                SessionExerciseIden::SessionId,
                SessionExerciseIden::CreationDate,
                SessionExerciseIden::LastUpdatedDate,
            ])
            .from(SessionExerciseIden::Table)
            .and_where(Expr::col(SessionExerciseIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_row(&*values.as_params(), SessionExercise::from_row)?;
        Ok(res)
    }
}