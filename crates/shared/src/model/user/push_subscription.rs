#[cfg(feature = "backend")]
use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
    ToSql,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushNotificationSubscription {
    pub endpoint: String,
    /// p256dh key
    pub key: String,
    pub auth: String,
}

#[cfg(feature = "backend")]
impl PushNotificationSubscription {
    fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(feature = "backend")]
impl ToSql for PushNotificationSubscription {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.to_json_string()
            .map(ToSqlOutput::from)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    }
}

#[cfg(feature = "backend")]
impl FromSql for PushNotificationSubscription {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        <serde_json::Value as FromSql>::column_result(value)
            .and_then(|v| serde_json::from_value(v).map_err(|e| FromSqlError::Other(Box::new(e))))
    }
}

#[cfg(feature = "backend")]
impl sea_query::Nullable for PushNotificationSubscription {
    fn null() -> sea_query::Value {
        sea_query::Value::String(None)
    }
}

#[cfg(feature = "backend")]
impl From<PushNotificationSubscription> for sea_query::Value {
    fn from(value: PushNotificationSubscription) -> Self {
        value
            .to_json_string()
            // TODO: this sucks
            .expect("Serialize PushNotificationSubscription")
            .into()
    }
}
