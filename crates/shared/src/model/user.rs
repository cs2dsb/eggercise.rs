use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature="database")]
use {
    exemplar::Model,
    rusqlite::Connection,
    sea_query::{enum_def, Expr, Query, SqliteQueryBuilder},
    sea_query_rusqlite::RusqliteBinder,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature="database", derive(Model))]
#[cfg_attr(feature="database", table("user"))]
#[cfg_attr(feature="database", check("../../../server/migrations/001-user/up.sql"))]
#[cfg_attr(feature="database", enum_def)]
pub struct User {
    id: i64,
    name: String,
    first_login: DateTime<Utc>,
    latest_login: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature="database", derive(Model))]
#[cfg_attr(feature="database", table("user"))]
pub struct NewUser {
    name: String,
}

#[cfg(feature="database")]
impl User {
    pub fn fetch(conn: &Connection, id: i64) -> Result<User, anyhow::Error> {
        let (sql, values) = Query::select()
            .columns([
                UserIden::Id,
                UserIden::Name,
                UserIden::FirstLogin,
                UserIden::LatestLogin
            ])
            .from(UserIden::Table)
            .and_where(Expr::col(UserIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let user = stmt.query_row(&*values.as_params(), User::from_row)?;
        Ok(user)
    }

    pub fn create(conn: &mut Connection, new_user: NewUser) -> Result<User, anyhow::Error> {
        let tx = conn.transaction()?;
        let user = {
            new_user.insert(&tx)?;
            User::fetch(&tx, tx.last_insert_rowid())?
        };
        tx.commit()?;

        Ok(user)
    }
}