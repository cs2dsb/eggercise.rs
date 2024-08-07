use chrono::{DateTime, Utc};

use crate::{api::error::ServerError, feature_model_derives, feature_model_imports, types::Uuid};

feature_model_imports!();

use std::{
    error::Error,
    ops::{Deref, DerefMut},
};

use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
    ToSql,
};
use webauthn_rs::prelude::{CredentialID as WebauthnCredentialId, Passkey as WebauthnPasskey};

/// Wrapper to implement ToSql and FromSql on
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Passkey(WebauthnPasskey);

impl Passkey {
    fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl From<WebauthnPasskey> for Passkey {
    fn from(value: WebauthnPasskey) -> Self {
        Self(value)
    }
}

impl Deref for Passkey {
    type Target = WebauthnPasskey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Passkey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ToSql for Passkey {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.to_json_string()
            .map(ToSqlOutput::from)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    }
}

impl FromSql for Passkey {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        <serde_json::Value as FromSql>::column_result(value)
            .and_then(|v| serde_json::from_value(v).map_err(|e| FromSqlError::Other(Box::new(e))))
    }
}

/// Wrapper to implement ToSql and FromSql on
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CredentialId(WebauthnCredentialId);

impl CredentialId {
    fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Deref for CredentialId {
    type Target = WebauthnCredentialId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<WebauthnCredentialId> for CredentialId {
    fn from(value: WebauthnCredentialId) -> Self {
        Self(value)
    }
}

impl ToSql for CredentialId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.to_json_string()
            .map(ToSqlOutput::from)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    }
}

impl FromSql for CredentialId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        <serde_json::Value as FromSql>::column_result(value)
            .and_then(|v| serde_json::from_value(v).map_err(|e| FromSqlError::Other(Box::new(e))))
    }
}

feature_model_derives!(
    "credential",
    "../../migrations/002-credential/up.sql",
    pub struct Credential {
        pub id: CredentialId,
        pub user_id: Uuid,
        pub passkey: Passkey,
        pub counter: u32,
        pub creation_date: DateTime<Utc>,
        pub last_used_date: Option<DateTime<Utc>>,
        pub last_updated_date: DateTime<Utc>,
        pub backup_eligible: bool,
        pub backup_state: bool,
    }
);

impl Credential {
    pub fn fetch<T: Error>(
        conn: &Connection,
        id: &CredentialId,
    ) -> Result<Credential, ServerError<T>> {
        let id_value = id.to_json_string()?;
        let (sql, values) = Self::select_star()
            .and_where(Expr::col(CredentialIden::Id).eq(id_value))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let credential = stmt.query_row(&*values.as_params(), Credential::from_row)?;

        Ok(credential)
    }

    pub fn fetch_passkeys<T: Error>(
        conn: &Connection,
        user_id: &Uuid,
    ) -> Result<Vec<WebauthnPasskey>, ServerError<T>> {
        let (sql, values) = Query::select()
            .column(CredentialIden::Passkey)
            .from(CredentialIden::Table)
            .and_where(Expr::col(CredentialIden::UserId).eq(user_id))
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let passkeys = stmt
            .query_and_then(&*values.as_params(), |r| r.get::<_, serde_json::Value>(0))?
            .map(|r| {
                r.map_err(ServerError::from)
                    .and_then(|v| serde_json::from_value(v).map_err(ServerError::from))
            })
            .collect::<Result<Vec<WebauthnPasskey>, _>>()?;

        Ok(passkeys)
    }

    #[cfg(feature = "exemplar-model")]
    pub fn create<T: Error>(
        conn: &mut Connection,
        new_credential: NewCredential,
    ) -> Result<Credential, ServerError<T>> {
        let tx = conn.transaction()?;
        let credential = {
            new_credential.insert(&tx)?;
            Credential::fetch(&tx, &new_credential.id)?
        };
        tx.commit()?;

        Ok(credential)
    }

    pub fn update<T: Error>(&self, conn: &Connection) -> Result<(), ServerError<T>> {
        let id_value = self.id.to_json_string()?;
        let (sql, values) = Query::update()
            .table(CredentialIden::Table)
            .values([
                (CredentialIden::Counter, self.counter.into()),
                (CredentialIden::LastUsedDate, self.last_used_date.into()),
                (CredentialIden::LastUpdatedDate, self.last_updated_date.into()),
                (CredentialIden::BackupState, self.backup_state.into()),
                (CredentialIden::BackupEligible, self.backup_eligible.into()),
            ])
            .and_where(Expr::col(CredentialIden::Id).eq(id_value))
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        stmt.execute(&*values.as_params())?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ExemplarModel)]
#[table("credential")]
pub struct NewCredential {
    pub id: CredentialId,
    pub user_id: Uuid,
    pub passkey: Passkey,
}

impl NewCredential {
    pub fn new<I: Into<Uuid>>(user_id: I, passkey: Passkey) -> Self {
        let id = passkey.cred_id().clone().into();
        let user_id = user_id.into();
        Self { id, user_id, passkey }
    }
}
