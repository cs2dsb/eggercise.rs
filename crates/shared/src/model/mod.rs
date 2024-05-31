mod user;
pub use user::*;

mod exercise;
pub use exercise::*;

#[cfg(feature = "backend")]
mod credential;
#[cfg(feature = "backend")]
pub use credential::*;

use crate::api::error::ValidationError;

pub mod constants;

mod temporary_login;
pub use temporary_login::*;

pub trait ValidateModel {
    fn validate(&self) -> Result<(), ValidationError>;
}

#[macro_export]
macro_rules! feature_model_imports {
    () => {
        #[cfg(feature = "exemplar-model")]
        #[allow(unused_imports)]
        use exemplar::Model;
        #[cfg(feature = "sea-query-enum")]
        #[allow(unused_imports)]
        use sea_query::{enum_def, Expr, Query, SqliteQueryBuilder};
        #[allow(unused_imports)]
        use serde::{Deserialize, Serialize};
        #[cfg(feature = "backend")]
        #[allow(unused_imports)]
        use {rusqlite::Connection, sea_query_rusqlite::RusqliteBinder};
    };
}

#[macro_export]
macro_rules! feature_model_derives {
    ($table_name:literal, $migration_path:literal, $($struct_body_tt:tt)*) => {

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[cfg_attr(feature = "exemplar-model", derive(Model))]
        #[cfg_attr(feature = "exemplar-model", table($table_name))]
        #[cfg_attr(
            feature = "exemplar-model",
            check($migration_path))]
        #[cfg_attr(feature = "sea-query-enum", enum_def)]
        $($struct_body_tt)*
    };
}
