#![allow(dead_code)]

use std::{any::type_name, collections::HashMap};

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use gloo_utils::{
    errors::{JsError, NotJsError},
    format::JsValueSerdeExt,
};
use leptos::{provide_context, use_context};
use sea_query::types::Iden;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use shared::types::UuidError;
use thiserror::Error;
use tracing::{error, trace};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::{self, Function, Promise};

#[derive(Debug, Clone, Error)]
pub enum SqlitePromiserError {
    #[error("Error getting promise from promiser: {0}")]
    Promiser(String),

    #[error("Error from sqlite (calling promiser promise): {0}")]
    Sqlite(String),

    #[error("Error de/serializing json: {0}")]
    Json(String),

    #[error("Unexpected result. Expected {0:?} but got {1:?}")]
    UnexpectedResult(Type, Type),

    #[error("Unexpected exec result: {0}")]
    ExecResult(String),

    #[error("JsValue wasn't an Error...: {0}")]
    NotJs(String),

    #[error("Column {0} was missing from ExecResult")]
    MissingColumn(String),

    #[error("Row {0} requested but only {1} rows in ExecResult")]
    MissingRow(usize, usize),

    #[error("Error parsing Uuid: {0}")]
    Uuid(UuidError),

    #[error("Error parsing Datetime: {0}")]
    Chrono(chrono::ParseError),
}

impl From<chrono::ParseError> for SqlitePromiserError {
    fn from(value: chrono::ParseError) -> Self {
        Self::Chrono(value)
    }
}

impl From<UuidError> for SqlitePromiserError {
    fn from(value: UuidError) -> Self {
        Self::Uuid(value)
    }
}
impl From<NotJsError> for SqlitePromiserError {
    fn from(value: NotJsError) -> Self {
        Self::NotJs(value.to_string())
    }
}

impl From<serde_json::Error> for SqlitePromiserError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}

impl SqlitePromiserError {
    fn from_promiser(value: JsValue) -> Self {
        match JsError::try_from(value) {
            Ok(v) => Self::Promiser(v.to_string()),
            Err(e) => Self::NotJs(e.to_string()),
        }
    }

    fn from_sqlite(value: JsValue) -> Self {
        // Copied out of gloo_utils because their version panics when the non error
        // isn't a string
        // See <https://github.com/rustwasm/gloo/issues/477>
        match value.dyn_into::<js_sys::Error>() {
            Ok(error) => Self::Sqlite(JsError::from(error).to_string()),
            Err(js_value) => match js_value.dyn_into::<js_sys::JsString>() {
                Ok(string) => {
                    let js_to_string = String::from(string);
                    Self::NotJs(format!(
                        "JsValue wasn't JsError, was actually a string: {js_to_string}"
                    ))
                }
                Err(js_value) => Self::NotJs(format!(
                    "JsValue wasn't JsError or JsString: {:?}",
                    js_value
                )),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct SqlitePromiser {
    inner: Function,
}

/// Copied from rusqlite FromSql
pub fn parse_datetime(value: &str) -> Result<DateTime<Utc>, SqlitePromiserError> {
    {
        // Try to parse value as rfc3339 first.
        let fmt = if value.len() >= 11 && value.as_bytes()[10] == b'T' {
            "%FT%T%.f%#z"
        } else {
            "%F %T%.f%#z"
        };

        if let Ok(dt) = DateTime::parse_from_str(value, fmt) {
            return Ok(dt.with_timezone(&Utc));
        }
    }

    // Couldn't parse as rfc3339 - fall back to NaiveDateTime.
    let fmt = if value.len() >= 11 && value.as_bytes()[10] == b'T' {
        "%FT%T%.f"
    } else {
        "%F %T%.f"
    };

    Ok(NaiveDateTime::parse_from_str(value, fmt).map(|dt| Utc.from_utc_datetime(&dt))?)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
enum Args {
    None,
    Sql(ExecArgs),
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ExecArgs {
    sql: String,
    result_rows: Vec<serde_json::Value>,
    column_names: Vec<serde_json::Value>,
}

impl<T: Into<String>> From<T> for ExecArgs {
    fn from(value: T) -> Self {
        ExecArgs {
            sql: value.into(),
            result_rows: Vec::new(),
            column_names: Vec::new(),
        }
    }
}

fn is_none(args: &Args) -> bool {
    *args == Args::None
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Type {
    ConfigGet,
    Exec,
    OpfsTree,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Command {
    #[serde(rename = "type")]
    pub type_: Type,
    #[serde(skip_serializing_if = "is_none")]
    pub args: Args,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandResult {
    pub db_id: String,
    pub message_id: String,
    pub worker_received_time: f32,
    pub worker_respond_time: f32,
    pub departure_time: f32,

    #[serde(flatten)]
    pub result: InnerResult,

    #[serde(flatten)]
    pub extra_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigGetResult {
    pub big_int_enabled: bool,
    pub version: Version,
    pub vfs_list: Vec<String>,
    pub opfs_enabled: bool,

    #[serde(flatten)]
    pub extra_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecResult {
    pub sql: String,
    pub result_rows: Vec<Vec<serde_json::Value>>,
    pub column_names: Vec<String>,

    #[serde(flatten)]
    pub extra_fields: HashMap<String, serde_json::Value>,
}

impl ExecResult {
    pub fn get_extractor<T, J>(
        &self,
        column: T,
    ) -> Result<impl Fn(&Self, usize) -> Result<J, SqlitePromiserError>, SqlitePromiserError>
    where
        T: Iden,
        J: DeserializeOwned,
    {
        let column_name = column.to_string();
        let (column_index, _) = self
            .column_names
            .iter()
            .enumerate()
            .find(|(_, cn)| cn == &&column_name)
            .ok_or(SqlitePromiserError::MissingColumn(column_name.clone()))?;

        Ok(move |r: &Self, row_index| {
            let row = if r.result_rows.len() > row_index {
                &r.result_rows[row_index]
            } else {
                Err(SqlitePromiserError::MissingRow(
                    row_index,
                    r.result_rows.len(),
                ))?
            };

            let v = if row.len() > column_index {
                &row[column_index]
            } else {
                Err(SqlitePromiserError::MissingColumn(format!(
                    "{} column_names contained the column ({}) but it was missing from row {}",
                    type_name::<Self>(),
                    column_name,
                    row_index,
                )))?
            };

            let r = serde_json::from_value(v.clone())?;
            Ok(r)
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpfsTreeResults {
    pub dirs: Vec<String>,
    pub files: Vec<String>,
    pub name: String,

    #[serde(flatten)]
    pub extra_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "result")]
#[serde(rename_all = "kebab-case")]
enum InnerResult {
    ConfigGet(ConfigGetResult),
    Exec(ExecResult),
    OpfsTree(OpfsTreeResults),
}

impl InnerResult {
    fn type_(&self) -> Type {
        use InnerResult::*;
        match self {
            ConfigGet(_) => Type::ConfigGet,
            Exec(_) => Type::Exec,
            OpfsTree(_) => Type::OpfsTree,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub lib_version: String,
    pub lib_version_number: u64,
    pub source_id: String,
    pub download_version: u64,
}

impl SqlitePromiser {
    pub fn new(inner: Function) -> Self {
        Self {
            inner,
        }
    }

    pub fn provide_context(self) {
        provide_context(self);
    }

    pub fn use_promiser() -> Self {
        use_context::<Self>().expect(&format!("{} missing from context", type_name::<Self>()))
    }

    pub async fn configure(&self) -> Result<(), SqlitePromiserError> {
        let pragmas = r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
        "#;
        self.exec(pragmas).await?;
        Ok(())
    }

    async fn send_command(
        &self,
        type_: Type,
        args: Args,
    ) -> Result<CommandResult, SqlitePromiserError> {
        let this = JsValue::from(&self.inner);
        let cmd = Command {
            type_,
            args,
        };
        let cmd_value = <JsValue as JsValueSerdeExt>::from_serde(&cmd)?;
        trace!("Command: {:#?}", cmd_value);

        let promise: Promise = self
            .inner
            .call1(&this, &cmd_value)
            .map_err(SqlitePromiserError::from_promiser)?
            .into();
        let result: CommandResult = JsValueSerdeExt::into_serde(
            &JsFuture::from(promise)
                .await
                .map_err(SqlitePromiserError::from_sqlite)?,
        )?;
        trace!("Result: {:#?}", result);
        let ret_type = result.result.type_();

        if ret_type != type_ {
            Err(SqlitePromiserError::UnexpectedResult(
                Type::ConfigGet,
                result.result.type_(),
            ))
        } else {
            Ok(result)
        }
    }

    pub async fn get_config(&self) -> Result<ConfigGetResult, SqlitePromiserError> {
        let result = self.send_command(Type::ConfigGet, Args::None).await?;

        let InnerResult::ConfigGet(result) = result.result
        // The type is checked by send_command
        else {
            unreachable!()
        };

        Ok(result)
    }

    pub async fn exec<T: Into<String>>(&self, sql: T) -> Result<ExecResult, SqlitePromiserError> {
        let result = self
            .send_command(Type::Exec, Args::Sql(ExecArgs::from(sql)))
            .await?;

        let InnerResult::Exec(result) = result.result else {
            unreachable!()
        };

        Ok(result)
    }

    pub async fn get_value<T: Into<String>, V: DeserializeOwned>(
        &self,
        sql: T,
    ) -> Result<V, SqlitePromiserError> {
        let mut result = self.exec(sql).await?;

        if result.column_names.len() != 1 {
            Err(SqlitePromiserError::ExecResult(format!(
                "get_value expected a single column result but got {}",
                result.column_names.len()
            )))
        } else if result.result_rows.len() != 1 {
            Err(SqlitePromiserError::ExecResult(format!(
                "get_value expected a single row result but got {}",
                result.result_rows.len()
            )))
        } else if result.result_rows[0].len() != 1 {
            Err(SqlitePromiserError::ExecResult(format!("get_value expected a single row result with a single value inside but got {}. (This seems like a sqlite bug)", result.result_rows[0].len())))
        } else {
            let json_value = result.result_rows.pop().unwrap().pop().unwrap();

            let value = serde_json::from_value(json_value)?;
            Ok(value)
        }
    }

    pub async fn opfs_tree(&self) -> Result<OpfsTreeResults, SqlitePromiserError> {
        let result = self.send_command(Type::OpfsTree, Args::None).await?;

        let InnerResult::OpfsTree(result) = result.result else {
            unreachable!()
        };

        Ok(result)
    }
}
