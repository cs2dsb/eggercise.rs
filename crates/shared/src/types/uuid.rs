use std::{fmt, ops::Deref};

use serde::{Deserialize, Serialize};
pub use uuid::Error as UuidError;
#[cfg(feature = "exemplar-model")]
use {
    rusqlite::{
        types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
        ToSql,
    },
    std::str::FromStr,
};

/// Wrapper to implement ToSql and FromSql on
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Uuid(uuid::Uuid);

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<uuid::Uuid> for Uuid {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl Deref for Uuid {
    type Target = uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "exemplar-model")]
impl ToSql for Uuid {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(self.to_string().into()))
    }
}

#[cfg(feature = "exemplar-model")]
impl FromSql for Uuid {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        uuid::Uuid::from_str(value.as_str()?)
            .map(Uuid::from)
            .map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}

#[cfg(feature = "sea-query-enum")]
impl From<&Uuid> for sea_query::Value {
    fn from(value: &Uuid) -> Self {
        value.to_string().into()
    }
}

#[cfg(feature = "sea-query-enum")]
impl From<Uuid> for sea_query::Value {
    fn from(value: Uuid) -> Self {
        value.to_string().into()
    }
}

impl Uuid {
    pub fn new_v4() -> Self {
        uuid::Uuid::new_v4().into()
    }

    pub fn parse(value: &str) -> Result<Self, uuid::Error> {
        uuid::Uuid::parse_str(value).map(|v| v.into())
    }
}
