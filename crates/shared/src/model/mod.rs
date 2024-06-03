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
    const NUM_FIELDS: usize;
    type Iden: sea_query::Iden;
    fn iden_for_field(field: usize) -> Self::Iden;
    fn field_idens() -> &'static [Self::Iden];
    fn select_star() -> sea_query::SelectStatement;
    fn fetch_all_sql() -> String;
}

#[cfg(feature = "frontend")]
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
        use exemplar::Model;
        #[cfg(feature = "frontend")]
        #[allow(unused_imports)]
        use leptos::{view, IntoView};
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
                    Query::select()
                        .columns([
                            $(
                                [<$struct_name Iden>]::[<$field_name:camel>],
                            )*
                        ])
                        .from([<$struct_name Iden>]::Table)
                        .take()
                }

                fn fetch_all_sql() -> String {
                    Self::select_star().to_string(SqliteQueryBuilder)
                }
            }
        }

        #[cfg(feature = "frontend")]
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
