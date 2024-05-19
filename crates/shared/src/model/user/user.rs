use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::types::Uuid;


#[cfg(feature="backend")]
use {
    crate::model::NewUser,
    exemplar::Model,
    rusqlite::{Connection, OptionalExtension},
    sea_query::{enum_def, Expr, Query, SqliteQueryBuilder},
    sea_query_rusqlite::RusqliteBinder,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature="backend", derive(Model))]
#[cfg_attr(feature="backend", table("user"))]
#[cfg_attr(feature="backend", check("../../../../server/migrations/001-user/up.sql"))]
#[cfg_attr(feature="backend", enum_def)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub registration_date: DateTime<Utc>,
    pub last_updated_date: DateTime<Utc>,
    pub last_login_date: Option<DateTime<Utc>>,
}

#[cfg(feature="backend")]
impl User {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<User, anyhow::Error> {
        let (sql, values) = Query::select()
            .columns([
                UserIden::Id,
                UserIden::Username,
                UserIden::Email,
                UserIden::DisplayName,
                UserIden::RegistrationDate,
                UserIden::LastUpdatedDate,
                UserIden::LastLoginDate,
            ])
            .from(UserIden::Table)
            .and_where(Expr::col(UserIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let user = stmt.query_row(&*values.as_params(), User::from_row)?;
        Ok(user)
    }

    pub fn fetch_by_username<T: AsRef<str>>(conn: &Connection, username: T) -> Result<Option<User>, anyhow::Error> {
        let (sql, values) = Query::select()
            .columns([
                UserIden::Id,
                UserIden::Username,
                UserIden::Email,
                UserIden::DisplayName,
                UserIden::RegistrationDate,
                UserIden::LastUpdatedDate,
                UserIden::LastLoginDate,
            ])
            .from(UserIden::Table)
            .and_where(Expr::col(UserIden::Username).eq(username.as_ref()))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let user = stmt.query_row(&*values.as_params(), User::from_row).optional()?;
        Ok(user)
    }

    pub fn create(conn: &mut Connection, new_user: NewUser) -> Result<User, anyhow::Error> {
        let tx = conn.transaction()?;
        let user = {
            new_user.insert(&tx)?;
            User::fetch_by_id(&tx, &new_user.id)?
        };
        tx.commit()?;

        Ok(user)
    }

    pub fn update(&self, conn: &Connection) -> Result<(), anyhow::Error> {
        let (sql, values) = Query::update()
            .table(UserIden::Table)
            .values([
                (UserIden::Username, self.username.clone().into()),
                (UserIden::Email, self.email.clone().into()),
                (UserIden::DisplayName, self.display_name.clone().into()),
                (UserIden::RegistrationDate, self.registration_date.into()),
                (UserIden::LastUpdatedDate, self.last_updated_date.into()),
                (UserIden::LastLoginDate, self.last_login_date.into()),
            ])
            .and_where(Expr::col(UserIden::Id).eq(&self.id))
            .build_rusqlite(SqliteQueryBuilder);
        
        let mut stmt = conn.prepare_cached(&sql)?;
        stmt.execute(&*values.as_params())?;

        Ok(())
    }
}