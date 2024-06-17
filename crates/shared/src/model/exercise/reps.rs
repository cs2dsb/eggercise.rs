#[cfg(feature = "backend")]
use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
    ToSql,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Represesnts both a target number of reps and the actual number of reps
/// recorded
pub enum Reps {
    /// As many reps as possible
    /// For target reps, the contained number is the minimum (usually the same
    /// number as the previous non-amrap sets) For actual reps, the
    /// contained number is the number achieved
    Amrap(u32),
    /// Standard rep target
    Reps(u32),
}

#[cfg(feature = "backend")]
impl ToSql for Reps {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        serde_json::to_string_pretty(self)
            .map(ToSqlOutput::from)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    }
}

#[cfg(feature = "backend")]
impl FromSql for Reps {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        <serde_json::Value as FromSql>::column_result(value)
            .and_then(|v| serde_json::from_value(v).map_err(|e| FromSqlError::Other(Box::new(e))))
    }
}
