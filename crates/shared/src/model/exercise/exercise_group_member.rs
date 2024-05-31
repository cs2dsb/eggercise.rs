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
#[cfg_attr(feature = "backend", table("exercise_group_member"))]
#[cfg_attr(
    feature = "backend",
    check("../../../migrations/007-exercise_group/up.sql")
)]
#[cfg_attr(feature = "backend", enum_def)]
pub struct ExerciseGroupMember {
    pub id:             Uuid,
    pub exercise_id:    Uuid,
    pub group_id:       Uuid,
}

#[cfg(feature = "backend")]
impl ExerciseGroupMember {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<ExerciseGroupMember, rusqlite::Error> {
        let (sql, values) = Query::select()
            .columns([
                ExerciseGroupMemberIden::Id,
                ExerciseGroupMemberIden::ExerciseId,
                ExerciseGroupMemberIden::GroupId,
            ])
            .from(ExerciseGroupMemberIden::Table)
            .and_where(Expr::col(ExerciseGroupMemberIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_row(&*values.as_params(), ExerciseGroupMember::from_row)?;
        Ok(res)
    }
}