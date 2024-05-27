#![allow(dead_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::trace;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::{Function, Promise};
use gloo_utils::{errors::{JsError, NotJsError}, format::JsValueSerdeExt};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SqlitePromiserError {
    #[error("Error getting promise from promiser: {0}")]
    Promiser(JsError),

    #[error("Error from sqlite (calling promiser promise): {0}")]
    Sqlite(JsError),

    #[error("Error serializing json: {0}")]
    // TODO: is this inflating the binary size for little benefit?
    Json(serde_json::Error),

    #[error("Unexpected result. Expected {0:?} but got {1:?}")]
    UnexpectedResult(Type, Type),

    #[error("JsValue wasn't an Error...: {0}")]
    NotJs(NotJsError),
}

impl From<NotJsError> for SqlitePromiserError {
    fn from(value: NotJsError) -> Self {
        Self::NotJs(value)
    }
}

impl From<serde_json::Error> for SqlitePromiserError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl SqlitePromiserError {
    fn from_promiser(value: JsValue) -> Self {
        match JsError::try_from(value) {
            Ok(v) => Self::Promiser(v),
            Err(e) => Self::NotJs(e),
        }
    }

    fn from_sqlite(value: JsValue) -> Self {
        match JsError::try_from(value) {
            Ok(v) => Self::Sqlite(v),
            Err(e) => Self::NotJs(e),
        }
    }
}

#[derive(Debug)]
pub struct SqlitePromiser {
    inner: Function,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all="kebab-case")]
#[serde(untagged)]
enum Args {
    None,
    Sql(ExecArgs),
}


#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all="camelCase")]
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
#[serde(rename_all="kebab-case")]
pub enum Type {
    ConfigGet,
    Exec,
    OpfsTree,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Command {
    #[serde(rename="type")]
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
#[serde(tag="type", content="result" )]
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
            inner
        }
    }

    async fn send_command(&self, type_: Type, args: Args) -> Result<CommandResult, SqlitePromiserError> {
        let this = JsValue::from(&self.inner); 
        let cmd = Command {
            type_,
            args,
        };
        let cmd_value = <JsValue as JsValueSerdeExt>::from_serde(&cmd)?;
        trace!("Command: {:#?}", cmd_value);
        
        let promise: Promise = self.inner.call1(
            &this,
            &cmd_value)
            .map_err(SqlitePromiserError::from_promiser)?
            .into();

        let result: CommandResult = JsValueSerdeExt::into_serde(
            &JsFuture::from(promise).await
                .map_err(SqlitePromiserError::from_sqlite)?
            )?;

        trace!("Result: {:#?}", result);
        let ret_type = result.result.type_();

        if ret_type != type_ {
            Err(SqlitePromiserError::UnexpectedResult(Type::ConfigGet, result.result.type_()))
        } else {
            Ok(result)
        }
    }

    pub async fn get_config(&self) -> Result<ConfigGetResult, SqlitePromiserError> {
        let result = self.send_command(Type::ConfigGet, Args::None).await?;
            
        let InnerResult::ConfigGet(result) = result.result
        // The type is checked by send_command
        else { unreachable!() };
        
        Ok(result)
    }

    pub async fn exec<T: Into<String>>(&self, sql: T) -> Result<ExecResult, SqlitePromiserError> {
        let result = self.send_command(Type::Exec, Args::Sql(ExecArgs::from(sql))).await?;

        let InnerResult::Exec(result) = result.result
        else { unreachable!() };

        Ok(result)
    }

    pub async fn opfs_tree(&self) -> Result<OpfsTreeResults, SqlitePromiserError> {
        let result = self.send_command(Type::OpfsTree, Args::None).await?;

        let InnerResult::OpfsTree(result) = result.result
        else { unreachable!() };

        Ok(result)
    }
}