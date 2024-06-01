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

#[cfg(feature = "sea-query-enum")]
pub trait Model {
    fn select_star() -> sea_query::SelectStatement;
    fn fetch_all_sql() -> String;
}

#[macro_export]
macro_rules! feature_model_imports {
    () => {
        #[cfg(feature = "exemplar-model")]
        #[allow(unused_imports)]
        use exemplar::Model;
        #[cfg(feature = "sea-query-enum")]
        #[allow(unused_imports)]
        use sea_query::{enum_def, Expr, Query, SelectStatement, SqliteQueryBuilder};
        #[allow(unused_imports)]
        use serde::{Deserialize, Serialize};
        #[cfg(feature = "backend")]
        #[allow(unused_imports)]
        use {rusqlite::Connection, sea_query_rusqlite::RusqliteBinder};

        #[cfg(feature = "sea-query-enum")]
        #[allow(unused_imports)]
        use crate::model::Model as _;
    };
}

#[macro_export]
macro_rules! feature_model_derives {
    ($table_name:literal, $migration_path:literal, pub struct $struct_name:ident {
        $(pub $field_name:ident: $field_type:ty,)*
    }) => {

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[cfg_attr(feature = "exemplar-model", derive(Model))]
        #[cfg_attr(feature = "exemplar-model", table($table_name))]
        #[cfg_attr(
            feature = "exemplar-model",
            check($migration_path))]
        #[cfg_attr(feature = "sea-query-enum", enum_def)]
        pub struct $struct_name {
            $(pub $field_name: $field_type,)*
        }

        #[cfg(feature = "sea-query-enum")]
        impl crate::model::Model for $struct_name {
            fn select_star() -> SelectStatement {
                paste::paste! {
                    Query::select()
                        .columns([
                            $(
                                [<$struct_name Iden>]::[<$field_name:camel>],
                            )*
                        ])
                        .from([<$struct_name Iden>]::Table)
                        .take()
                }
            }
            fn fetch_all_sql() -> String {
                Self::select_star().to_string(SqliteQueryBuilder)
            }
        }


        #[cfg(test)]
        paste::paste! {
            mod [<$struct_name:snake:lower _tests>] {
                #[allow(unused_imports)]
                use super::*;

                #[test]
                #[cfg(feature = "sea-query-enum")]
                fn [<test_ $struct_name:snake:lower _fetch_all_sql>]() {
                    let sql = $struct_name::fetch_all_sql()
                        .to_lowercase();

                    assert!(sql.starts_with("select "));

                    let mut fields = String::new();
                    $(
                        fields.push('"');
                        fields.push_str(stringify!($field_name));
                        fields.push_str("\", ");
                    )*
                    fields.truncate(fields.len() - 2);

                    assert!(sql.contains(&fields));
                }
            }
        }
    };
}
