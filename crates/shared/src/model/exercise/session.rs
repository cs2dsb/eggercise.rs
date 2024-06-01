use chrono::{DateTime, Utc};

use crate::{feature_model_derives, feature_model_imports, types::Uuid};

feature_model_imports!();

feature_model_derives!(
    "session",
    "../../../migrations/005-session/up.sql",
    pub struct Session {
        pub id: Uuid,
        pub creation_date: DateTime<Utc>,
        pub last_updated_date: DateTime<Utc>,
    }
);

#[cfg(feature = "backend")]
impl Session {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<Session, rusqlite::Error> {
        let (sql, values) = Self::select_star()
            .and_where(Expr::col(SessionIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_row(&*values.as_params(), Session::from_row)?;
        Ok(res)
    }
}
