use shared::{
    model::{ExerciseGroup, ExerciseGroupIden},
    types::Uuid,
};

use crate::{
    db::PromiserFetcher,
    utils::sqlite3::{parse_datetime, ExecResult, SqlitePromiserError},
};

impl PromiserFetcher for ExerciseGroup {
    fn extract_fields(result: ExecResult) -> Result<Vec<Self>, SqlitePromiserError> {
        let id_e = result.get_extractor(ExerciseGroupIden::Id)?;
        let name_e = result.get_extractor(ExerciseGroupIden::Name)?;
        let description_e = result.get_extractor(ExerciseGroupIden::Description)?;
        let creation_date_e = result.get_extractor(ExerciseGroupIden::CreationDate)?;
        let last_updated_date_e = result.get_extractor(ExerciseGroupIden::LastUpdatedDate)?;

        (0..result.result_rows.len())
            .into_iter()
            .map(|i| {
                let res = ExerciseGroup {
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
            .collect::<Result<Vec<_>, _>>()
    }
}
