use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::{api::error::ValidationError, types::Uuid};

use super::ValidateModel;

#[cfg(feature="backend")]
use {
    anyhow::Context,
    super::{ Credential, NewCredential },
    exemplar::Model,
    rusqlite::{Connection, OptionalExtension},
    sea_query::{enum_def, Expr, Query, SqliteQueryBuilder},
    sea_query_rusqlite::RusqliteBinder,
    webauthn_rs::prelude::Passkey,
};

const USERNAME_MIN_LENGTH: usize = 4;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature="backend", derive(Model))]
#[cfg_attr(feature="backend", table("user"))]
#[cfg_attr(feature="backend", check("../../../server/migrations/001-user/up.sql"))]
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature="backend", derive(Model))]
#[cfg_attr(feature="backend", table("user"))]
pub struct NewUser {
    pub id: Uuid,
    pub username: String,
}

impl NewUser {
    pub fn new<I: Into<Uuid>, T: Into<String>>(id: I, username: T) -> Self {
        Self {
            id: id.into(),
            username: username.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrationUser {
    pub username: String,
}

impl RegistrationUser {
    pub fn new<T: Into<String>>(username: T) -> Self {
        let username = username.into();
        Self {
            username
        }
    }
}

impl ValidateModel for RegistrationUser {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.username.len() < USERNAME_MIN_LENGTH {
            Err(ValidationError { 
                error_messages: vec![
                    format!("Username needs to be at least {USERNAME_MIN_LENGTH} characters long"),
            ]})
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg(feature="backend")]
pub struct NewUserWithPasskey {
    pub id: Uuid,
    pub username: String,
    pub passkey: Passkey,
}

#[cfg(feature="backend")]
impl NewUserWithPasskey {
    fn split(self) -> (NewUser, Passkey) {
        let Self { id, username, passkey } = self;
        (
            NewUser::new(id, username),
            passkey,
        )
    }
    pub fn new<I: Into<Uuid>, T: Into<String>>(id: I, username: T, passkey: Passkey) -> Self {
        Self {
            id: id.into(),
            username: username.into(),
            passkey,
        }
    }

    pub fn create(self, conn: &mut Connection) -> Result<(User, Credential), anyhow::Error> {
        let tx = conn.transaction()?;

        let (new_user, passkey) = self.split();
        let user_id = new_user.id.clone();
        let new_credential = NewCredential::new(new_user.id.clone(), passkey.into());

        let user = {
            new_user.insert(&tx)
                .context("NewUserWithPasskey::insert(User)")?;

            User::fetch_by_id(&tx, &user_id)
                .context("NewUserWithPasskey::fetch(User)")?
        };

        let credential = {
            new_credential.insert(&tx)
                .context("NewUserWithPasskey::insert(Credential)")?;
            Credential::fetch(&tx, &new_credential.id)
                .context("NewUserWithPasskey::fetch(Credential)")?
        };

        tx.commit()?;

        Ok((user, credential))
    }
}