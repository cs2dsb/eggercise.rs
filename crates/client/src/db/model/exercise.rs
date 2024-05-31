use leptos::{create_local_resource, Resource};
use shared::{
    model::{Exercise, ExerciseIden},
    types::Uuid,
};

use crate::utils::sqlite3::{parse_datetime, SqlitePromiser, SqlitePromiserError};

pub fn get_exercises() -> Resource<(), Result<Vec<Exercise>, SqlitePromiserError>> {
    create_local_resource(
        || (),
        |_| async {
            let promiser = SqlitePromiser::use_promiser();

            let result = promiser
                .exec(shared::model::Exercise::fetch_all_get_sql())
                .await?;

            let id_e = result.get_extractor(ExerciseIden::Id)?;
            let name_e = result.get_extractor(ExerciseIden::Name)?;
            let description_e = result.get_extractor(ExerciseIden::Description)?;
            let creation_date_e = result.get_extractor(ExerciseIden::CreationDate)?;
            let last_updated_date_e = result.get_extractor(ExerciseIden::LastUpdatedDate)?;

            let rows = (0..result.result_rows.len())
                .into_iter()
                .map(|i| {
                    let res = Exercise {
                        id: id_e(&result, i).and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                        name: name_e(&result, i)?,
                        description: description_e(&result, i)?,
                        creation_date: creation_date_e(&result, i)
                            .and_then(|s: String| Ok(parse_datetime(&s)?))?,
                        last_updated_date: last_updated_date_e(&result, i)
                            .and_then(|s: String| Ok(parse_datetime(&s)?))?,
                    };

                    Ok::<_, SqlitePromiserError>(res)
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(rows)
        },
    )
}
