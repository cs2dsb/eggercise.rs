use leptos::{create_local_resource, Resource};

use crate::utils::sqlite3::{SqlitePromiser, SqlitePromiserError};

#[derive(Debug, Clone)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    pub creation_date: String,
    pub last_updated_date: String,
}

pub fn get_exercises() -> Resource<(), Result<Vec<Exercise>, SqlitePromiserError>> {
    create_local_resource(
        || (),
        |_| {
            async {
                let promiser = SqlitePromiser::use_promiser();

                let result = promiser
                    .exec("SELECT id, name, creation_date, last_updated_date FROM exercise")
                    .await?;
                // TODO: automate this kind of check
                if result.column_names.len() != 4
                    || result.column_names[0] != "id"
                    || result.column_names[1] != "name"
                    || result.column_names[2] != "creation_date"
                    || result.column_names[3] != "last_updated_date"
                {
                    Err(SqlitePromiserError::ExecResult(format!(
                        "Expected id, name, creation_date, last_updated_date but got {:?}",
                        result.column_names
                    )))?
                }

                let rows = result
                    .result_rows
                    .into_iter()
                    .map(|mut r| {
                        Ok::<_, serde_json::Error>(Exercise {
                            id: serde_json::from_value(r.remove(0))?,
                            name: serde_json::from_value(r.remove(0))?,
                            creation_date: serde_json::from_value(r.remove(0))?,
                            last_updated_date: serde_json::from_value(r.remove(0))?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(rows)
            }
        },
    )
}
