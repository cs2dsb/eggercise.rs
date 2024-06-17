use std::ops::{Deref, DerefMut};

#[cfg(feature = "backend")]
use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
    ToSql,
};
use serde::{Deserialize, Serialize};

use super::{Reps, Weight};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Set {
    weight: Weight,
    reps: Reps,
    notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sets(pub Vec<Set>);

impl Deref for Sets {
    type Target = Vec<Set>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Sets {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "backend")]
impl ToSql for Set {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        serde_json::to_string_pretty(self)
            .map(ToSqlOutput::from)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    }
}

#[cfg(feature = "backend")]
impl FromSql for Set {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        <serde_json::Value as FromSql>::column_result(value)
            .and_then(|v| serde_json::from_value(v).map_err(|e| FromSqlError::Other(Box::new(e))))
    }
}

#[cfg(feature = "backend")]
impl ToSql for Sets {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        serde_json::to_string_pretty(self)
            .map(ToSqlOutput::from)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    }
}

#[cfg(feature = "backend")]
impl FromSql for Sets {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        <serde_json::Value as FromSql>::column_result(value)
            .and_then(|v| serde_json::from_value(v).map_err(|e| FromSqlError::Other(Box::new(e))))
    }
}
