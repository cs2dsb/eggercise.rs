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
#[cfg_attr(feature = "backend", table("session"))]
#[cfg_attr(
    feature = "backend",
    check("../../../migrations/005-session/up.sql")
)]
#[cfg_attr(feature = "backend", enum_def)]
pub struct Session {
    pub id:                  Uuid,
    pub creation_date:       DateTime<Utc>,
    pub last_updated_date:   DateTime<Utc>,
}

#[cfg(feature = "backend")]
impl Session {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<Session, rusqlite::Error> {
        let (sql, values) = Query::select()
            .columns([
                SessionIden::Id,
                SessionIden::CreationDate,
                SessionIden::LastUpdatedDate,
            ])
            .from(SessionIden::Table)
            .and_where(Expr::col(SessionIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_row(&*values.as_params(), Session::from_row)?;
        Ok(res)
    }
}