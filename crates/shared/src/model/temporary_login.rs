use chrono::{DateTime, Utc};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
#[cfg(feature = "backend")]
use {
    crate::api::error::ServerError,
    exemplar::Model,
    rusqlite::{ Connection, OptionalExtension },
    sea_query::{enum_def, Expr, Query, SqliteQueryBuilder},
    sea_query_rusqlite::RusqliteBinder,
    std::error::Error,
};

use crate::{api::Object, types::Uuid};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "backend", derive(Model))]
#[cfg_attr(feature = "backend", table("temporary_login"))]
#[cfg_attr(
    feature = "backend",
    check("../../../server/migrations/003-temporary_login/up.sql")
)]
#[cfg_attr(feature = "backend", enum_def)]
pub struct TemporaryLogin {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expiry_date: DateTime<Utc>,
    pub url: String,
}

impl TemporaryLogin {
    pub fn qr_code_url(&self) -> String {
        Object::QrCode.id_path()
            .replace(":id", &percent_encode(self.url.as_bytes(), NON_ALPHANUMERIC).to_string())
    }

    pub fn expired(&self) -> bool {
        self.expiry_date < Utc::now()
    }
}

#[cfg(feature = "backend")]
impl TemporaryLogin {
    pub fn fetch<T: Error>(
        conn: &Connection,
        id: &Uuid,
    ) -> Result<TemporaryLogin, ServerError<T>> {
        let (sql, values) = Query::select()
            .columns([
                TemporaryLoginIden::Id,
                TemporaryLoginIden::UserId,
                TemporaryLoginIden::ExpiryDate,
                TemporaryLoginIden::Url,
            ])
            .from(TemporaryLoginIden::Table)
            .and_where(Expr::col(TemporaryLoginIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let temporary_login = stmt.query_row(&*values.as_params(), TemporaryLogin::from_row)?;

        Ok(temporary_login)
    }

    pub fn fetch_maybe<T: Error>(
        conn: &Connection,
        id: &Uuid,
    ) -> Result<Option<TemporaryLogin>, ServerError<T>> {
        let (sql, values) = Query::select()
            .columns([
                TemporaryLoginIden::Id,
                TemporaryLoginIden::UserId,
                TemporaryLoginIden::ExpiryDate,
                TemporaryLoginIden::Url,
            ])
            .from(TemporaryLoginIden::Table)
            .and_where(Expr::col(TemporaryLoginIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let temporary_login = stmt.query_row(&*values.as_params(), TemporaryLogin::from_row)
            .optional()?;

        Ok(temporary_login)
    }

    pub fn fetch_by_user_id<T: Error>(
        conn: &Connection,
        id: &Uuid,
    ) -> Result<Option<TemporaryLogin>, ServerError<T>> {
        let (sql, values) = Query::select()
            .columns([
                TemporaryLoginIden::Id,
                TemporaryLoginIden::UserId,
                TemporaryLoginIden::ExpiryDate,
                TemporaryLoginIden::Url,
            ])
            .from(TemporaryLoginIden::Table)
            .and_where(Expr::col(TemporaryLoginIden::UserId).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let temporary_login = stmt.query_row(&*values.as_params(), TemporaryLogin::from_row).optional()?;

        Ok(temporary_login)
    }

    pub fn create<T: Error>(
        conn: &mut Connection,
        temporary_login: TemporaryLogin,
    ) -> Result<TemporaryLogin, ServerError<T>> {
        let tx = conn.transaction()?;
        let temporary_login = {
            temporary_login.insert(&tx)?;
            TemporaryLogin::fetch(&tx, &temporary_login.id)?
        };
        tx.commit()?;

        Ok(temporary_login)
    }

    pub fn delete<T: Error>(&self, conn: &Connection) -> Result<(), ServerError<T>> {
        let (sql, values) = Query::delete()
            .from_table(TemporaryLoginIden::Table)
            .and_where(Expr::col(TemporaryLoginIden::Id).eq(&self.id))
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        stmt.execute(&*values.as_params())?;

        Ok(())
    }
}