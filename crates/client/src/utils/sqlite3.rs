#![allow(dead_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize)]
struct Args {

}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Type {
    #[serde(rename="config-get")]
    ConfigGet,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Command {
    #[serde(rename="type")]
    pub type_: Type,
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
#[serde(tag="type", content="result" )]
#[serde(rename_all = "kebab-case")]
enum InnerResult {
    ConfigGet(ConfigGetResult),
}

impl InnerResult {
    fn type_(&self) -> Type {
        use InnerResult::*;
        match self {
            ConfigGet(_) => Type::ConfigGet,
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

    pub async fn get_config(&self) -> Result<ConfigGetResult, SqlitePromiserError> {
        let this = JsValue::from(&self.inner);
        let cmd = Command {
            type_: Type::ConfigGet,
            args: Args {},
        };

        let cmd_value = <JsValue as JsValueSerdeExt>::from_serde(&cmd)?;
        let promise: Promise = self.inner.call1(
            &this,
            &cmd_value)
            .map_err(SqlitePromiserError::from_promiser)?
            .into();

        let fut = JsFuture::from(promise);
        
        let result: CommandResult = JsValueSerdeExt::into_serde(
            &fut.await
                .map_err(SqlitePromiserError::from_sqlite)?
            )?;
            
        if let InnerResult::ConfigGet(result) = result.result {
            Ok(result)
        } else {
            Err(SqlitePromiserError::UnexpectedResult(Type::ConfigGet, result.result.type_()))
        }
    }
}