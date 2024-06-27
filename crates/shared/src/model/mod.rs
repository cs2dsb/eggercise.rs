mod user;
pub use user::*;

mod exercise;
pub use exercise::*;

mod plan;
pub use plan::*;

mod service_version;
pub use service_version::*;

#[cfg(feature = "backend")]
mod credential;
#[cfg(feature = "backend")]
pub use credential::*;

use crate::api::error::ValidationError;

pub mod constants;

pub trait ValidateModel {
    fn validate(&self) -> Result<(), ValidationError>;
}

#[cfg(feature = "sea-query-enum")]
pub trait Model: Sized {
    const NUM_FIELDS: usize;
    type Iden: sea_query::Iden;
    fn iden_for_field(field: usize) -> Self::Iden;
    fn field_idens() -> &'static [Self::Iden];
    fn select_star() -> sea_query::SelectStatement;
    /// Gets the insert query without values set
    fn insert_query() -> sea_query::InsertStatement;
    #[cfg(feature = "wasm")]
    fn fetch_all_sql() -> String;
    #[cfg(feature = "wasm")]
    fn fetch_by_column_sql<T: Into<sea_query::Value>>(
        id: T,
        column: Self::Iden,
        limit_1: bool,
    ) -> String;
    #[cfg(feature = "backend")]
    fn fetch_all(conn: &rusqlite::Connection) -> Result<Vec<Self>, rusqlite::Error>;
    #[cfg(feature = "backend")]
    fn fetch_by_id<T: Into<sea_query::Value>>(
        conn: &rusqlite::Connection,
        id: T,
    ) -> Result<Self, rusqlite::Error>;
    #[cfg(feature = "backend")]
    fn fetch_by_column<T: Into<sea_query::Value>>(
        conn: &rusqlite::Connection,
        id: T,
        column: Self::Iden,
    ) -> Result<Self, rusqlite::Error>;
    #[cfg(feature = "backend")]
    fn fetch_by_id_maybe<T: Into<sea_query::Value>>(
        conn: &rusqlite::Connection,
        id: T,
    ) -> Result<Option<Self>, rusqlite::Error>;
    #[cfg(feature = "backend")]
    fn fetch_by_column_maybe<T: Into<sea_query::Value>>(
        conn: &rusqlite::Connection,
        id: T,
        column: Self::Iden,
    ) -> Result<Option<Self>, rusqlite::Error>;
    #[cfg(feature = "backend")]
    fn fetch_all_by_column<T: Into<sea_query::Value>>(
        conn: &rusqlite::Connection,
        id: T,
        column: Self::Iden,
    ) -> Result<Vec<Self>, rusqlite::Error>;
}

#[cfg(feature = "wasm")]
pub mod model_into_view {
    use leptos::{
        html::{table, tr, AnyElement},
        view, HtmlElement, IntoView,
    };

    use super::*;

    pub trait ModelIntoView: Model {
        fn container() -> HtmlElement<AnyElement> {
            table().into_any()
        }

        fn header_container() -> HtmlElement<AnyElement> {
            tr().into_any()
        }

        fn row_container() -> HtmlElement<AnyElement> {
            tr().into_any()
        }

        fn header_str(iden: <Self as Model>::Iden) -> &'static str;

        fn header_view(iden: <Self as Model>::Iden) -> impl IntoView {
            view! {
                <td> { Self::header_str(iden) } </td>
            }
        }

        fn row_view(&self, iden: <Self as Model>::Iden) -> impl IntoView;
    }

    pub trait DefaultModelIntoView: Model {
        fn header_str(iden: <Self as Model>::Iden) -> &'static str;
        fn row_view(&self, iden: <Self as Model>::Iden) -> impl IntoView;
    }

    /// Marker trait to manually apply to any Model instances you want to use
    /// the fallback rendering on
    pub trait UseDefaultModelView: DefaultModelIntoView {}

    impl<T: UseDefaultModelView> ModelIntoView for T {
        fn header_str(iden: <Self as Model>::Iden) -> &'static str {
            <Self as DefaultModelIntoView>::header_str(iden)
        }

        fn row_view(&self, iden: <Self as Model>::Iden) -> impl IntoView {
            <Self as DefaultModelIntoView>::row_view(self, iden)
        }
    }

    #[derive(Debug, Clone)]
    pub struct ListOfModel<T: ModelIntoView + Clone>(pub Vec<T>);

    impl<T: ModelIntoView + Clone> From<Vec<T>> for ListOfModel<T> {
        fn from(value: Vec<T>) -> Self {
            Self(value)
        }
    }

    impl<T: ModelIntoView + Clone> IntoView for &ListOfModel<T> {
        fn into_view(self) -> leptos::View {
            let mut container = T::container();

            // Create the header
            let mut header_container = T::header_container();
            for i in 0..T::NUM_FIELDS {
                let iden = T::iden_for_field(i);
                let column_header = T::header_view(iden);
                header_container = header_container.child(column_header);
            }
            container = container.child(header_container);

            // Go through the rows
            for row in self.0.iter() {
                let mut row_container = T::row_container();
                for i in 0..T::NUM_FIELDS {
                    let iden = T::iden_for_field(i);
                    let value = row.row_view(iden);
                    row_container = row_container.child(value);
                }
                container = container.child(row_container);
            }

            container.into_view()
        }
    }
}

#[macro_export]
macro_rules! feature_model_imports {
    () => {
        #[cfg(feature = "exemplar-model")]
        #[allow(unused_imports)]
        use exemplar::Model as ExemplarModel;
        #[cfg(feature = "wasm")]
        #[allow(unused_imports)]
        use leptos::{view, IntoView};
        #[cfg(feature = "sea-query-enum")]
        #[allow(unused_imports)]
        use sea_query::{
            enum_def, Expr, InsertStatement, Query, SelectStatement, SqliteQueryBuilder,
        };
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
    ($table_name:literal, $migration_path:literal, $(#[$($attrs:tt)*])* pub struct $struct_name:ident {
        $($(#[$($fattrs:tt)*])* pub $field_name:ident: $field_type:ty,)*
    }) => {

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[cfg_attr(feature = "exemplar-model", derive(ExemplarModel))]
        #[cfg_attr(feature = "exemplar-model", table($table_name))]
        #[cfg_attr(
            feature = "exemplar-model",
            check($migration_path))]
        #[cfg_attr(feature = "sea-query-enum", enum_def)]
        $(#[$($attrs)*])*
        pub struct $struct_name {
            $(
                $(#[$($fattrs)*])*
                pub $field_name: $field_type,
            )*
        }

        #[cfg(feature = "sea-query-enum")]
        impl crate::model::Model for $struct_name {
            paste::paste! {
                type Iden = [<$struct_name Iden>];

                const NUM_FIELDS: usize = ${count($field_name)};

                fn iden_for_field(field: usize) -> Self::Iden {
                    match field {
                        $(
                            ${index()} => [<$struct_name Iden>]::[<$field_name:camel>],
                        )*
                        _ => [<$struct_name Iden>]::Table,
                    }
                }

                fn field_idens() -> &'static [Self::Iden] {
                    &[
                        $(
                            [<$struct_name Iden>]::[<$field_name:camel>],
                        )*
                    ]
                }

                fn select_star() -> SelectStatement {
                    let mut stmt = Query::select();
                    stmt
                        .columns([
                            $(
                                [<$struct_name Iden>]::[<$field_name:camel>],
                            )*
                        ])
                        .from([<$struct_name Iden>]::Table);
                    stmt
                }

                fn insert_query() -> InsertStatement {
                    let mut stmt = Query::insert();
                    stmt
                        .into_table([<$struct_name Iden>]::Table)
                        .columns([
                            $(
                                [<$struct_name Iden>]::[<$field_name:camel>],
                            )*
                        ]);
                    stmt
                }

                #[cfg(feature = "wasm")]
                fn fetch_all_sql() -> String {
                    Self::select_star().to_string(SqliteQueryBuilder)
                }

                #[cfg(feature = "wasm")]
                fn fetch_by_column_sql<T: Into<sea_query::Value>>(id: T, column: Self::Iden, limit_1: bool) -> String {
                    let mut stmt = Self::select_star();
                    stmt.and_where(Expr::col(column).eq(id.into()));

                    if limit_1 {
                        stmt.limit(1);
                    }

                    stmt.to_string(SqliteQueryBuilder)
                }

                #[cfg(feature = "backend")]
                fn fetch_all(conn: &rusqlite::Connection) -> Result<Vec<Self>, rusqlite::Error> {
                    let (sql, values) = Self::select_star()
                        .build_rusqlite(SqliteQueryBuilder);

                    let mut stmt = conn.prepare_cached(&sql)?;

                    let results = stmt
                        .query_and_then(&*values.as_params(), Self::from_row)?
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(results)
                }

                #[cfg(feature = "backend")]
                fn fetch_by_id<T: Into<sea_query::Value>>(conn: &rusqlite::Connection, id: T) -> Result<Self, rusqlite::Error> {
                    Self::fetch_by_column(conn, id, [<$struct_name Iden>]::Id)
                }

                #[cfg(feature = "backend")]
                fn fetch_by_column<T: Into<sea_query::Value>>(conn: &rusqlite::Connection, id: T, column: Self::Iden) -> Result<Self, rusqlite::Error> {
                    let (sql, values) = Self::select_star()
                        .and_where(Expr::col(column).eq(id.into()))
                        .limit(1)
                        .build_rusqlite(SqliteQueryBuilder);

                    let mut stmt = conn.prepare_cached(&sql)?;

                    let result = stmt
                        .query_row(&*values.as_params(), Self::from_row)?;

                    Ok(result)
                }

                #[cfg(feature = "backend")]
                fn fetch_by_id_maybe<T: Into<sea_query::Value>>(conn: &rusqlite::Connection, id: T) -> Result<Option<Self>, rusqlite::Error> {
                    Self::fetch_by_column_maybe(conn, id, [<$struct_name Iden>]::Id)
                }

                #[cfg(feature = "backend")]
                fn fetch_by_column_maybe<T: Into<sea_query::Value>>(conn: &rusqlite::Connection, id: T, column: Self::Iden) -> Result<Option<Self>, rusqlite::Error> {
                    use rusqlite::OptionalExtension;

                    let (sql, values) = Self::select_star()
                        .and_where(Expr::col(column).eq(id.into()))
                        .limit(1)
                        .build_rusqlite(SqliteQueryBuilder);

                    let mut stmt = conn.prepare_cached(&sql)?;

                    let result = stmt
                        .query_row(&*values.as_params(), Self::from_row)
                        .optional()?;

                    Ok(result)
                }

                #[cfg(feature = "backend")]
                fn fetch_all_by_column<T: Into<sea_query::Value>>(conn: &rusqlite::Connection, id: T, column: Self::Iden) -> Result<Vec<Self>, rusqlite::Error> {
                    let (sql, values) = Self::select_star()
                        .and_where(Expr::col(column).eq(id.into()))
                        .build_rusqlite(SqliteQueryBuilder);

                    let mut stmt = conn.prepare_cached(&sql)?;

                    let results = stmt
                        .query_and_then(&*values.as_params(), Self::from_row)?
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(results)
                }
            }
        }

        #[cfg(feature = "wasm")]
        impl crate::model::model_into_view::DefaultModelIntoView for $struct_name {
            fn header_str(iden: <Self as crate::model::Model>::Iden) -> &'static str {
                paste::paste! {
                    match iden {
                        [<$struct_name Iden>]::Table => stringify!($table_name),
                        $(
                            [<$struct_name Iden>]::[<$field_name:camel>] => stringify!([<$field_name:camel>]),
                        )*
                    }
                }
            }

            fn row_view(&self, iden: <Self as crate::model::Model>::Iden) -> impl leptos::IntoView {
                paste::paste! {
                    leptos::view! {
                        <td>
                            {
                                match iden {
                                    [<$struct_name Iden>]::Table => stringify!($table_name).to_string(),
                                    $(
                                        [<$struct_name Iden>]::[<$field_name:camel>] => format!("{:?}", &self.$field_name),
                                    )*
                                }
                            }
                        </td>
                    }
                }
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
