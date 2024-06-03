use leptos::{create_local_resource, Resource};
use shared::model::{
    model_into_view::{ListOfModel, ModelIntoView},
    Model,
};

use crate::utils::sqlite3::{ExecResult, SqlitePromiser, SqlitePromiserError};

pub mod migrations;
pub mod model;

pub(crate) trait PromiserFetcher: Model + Clone + ModelIntoView {
    fn all_resource() -> Resource<(), Result<ListOfModel<Self>, SqlitePromiserError>> {
        create_local_resource(
            || (),
            |_| async {
                let promiser = SqlitePromiser::use_promiser();

                let result = promiser.exec(Self::fetch_all_sql()).await?;

                let rows = Self::extract_fields(result)?;

                let model_rows = ListOfModel(rows);

                Ok(model_rows)
            },
        )
    }

    fn extract_fields(result: ExecResult) -> Result<Vec<Self>, SqlitePromiserError>;
}
