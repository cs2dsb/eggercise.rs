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
#[cfg_attr(feature = "backend", table("exercise_group"))]
#[cfg_attr(
    feature = "backend",
    check("../../../migrations/007-exercise_group/up.sql")
)]
#[cfg_attr(feature = "backend", enum_def)]
pub struct ExerciseGroup {
    pub id:                  Uuid,
    pub name:                String,
    pub description:         Option<String>,
    pub creation_date:       DateTime<Utc>,
    pub last_updated_date:   DateTime<Utc>,
}

#[cfg(feature = "backend")]
impl ExerciseGroup {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<ExerciseGroup, rusqlite::Error> {
        let (sql, values) = Query::select()
            .columns([
                ExerciseGroupIden::Id,
                ExerciseGroupIden::Name,
                ExerciseGroupIden::Description,
                ExerciseGroupIden::CreationDate,
                ExerciseGroupIden::LastUpdatedDate,
            ])
            .from(ExerciseGroupIden::Table)
            .and_where(Expr::col(ExerciseGroupIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_row(&*values.as_params(), ExerciseGroup::from_row)?;
        Ok(res)
    }
}