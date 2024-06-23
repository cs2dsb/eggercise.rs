use shared::{
    model::{Exercise, ExerciseIden},
    types::Uuid,
};

use crate::db::{
    sqlite3::{parse_datetime, ExecResult, SqlitePromiserError},
    PromiserFetcher,
};

impl PromiserFetcher for Exercise {
    fn extract_fields(result: ExecResult) -> Result<Vec<Self>, SqlitePromiserError> {
        let id_e = result.get_extractor(ExerciseIden::Id)?;
        let name_e = result.get_extractor(ExerciseIden::Name)?;
        let description_e = result.get_extractor(ExerciseIden::Description)?;
        let base_recovery_days_e = result.get_extractor(ExerciseIden::BaseRecoveryDays)?;
        let creation_date_e = result.get_extractor(ExerciseIden::CreationDate)?;
        let last_updated_date_e = result.get_extractor(ExerciseIden::LastUpdatedDate)?;

        (0..result.result_rows.len())
            .into_iter()
            .map(|i| {
                let res = Exercise {
                    id: id_e(&result, i).and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                    name: name_e(&result, i)?,
                    description: description_e(&result, i)?,
                    base_recovery_days: base_recovery_days_e(&result, i)?,
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
