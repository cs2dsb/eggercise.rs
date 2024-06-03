use shared::{
    model::{SessionExercise, SessionExerciseIden},
    types::Uuid,
};

use crate::{
    db::PromiserFetcher,
    utils::sqlite3::{parse_datetime, ExecResult, SqlitePromiserError},
};

impl PromiserFetcher for SessionExercise {
    fn extract_fields(result: ExecResult) -> Result<Vec<Self>, SqlitePromiserError> {
        let id_e = result.get_extractor(SessionExerciseIden::Id)?;
        let exercise_id_e = result.get_extractor(SessionExerciseIden::ExerciseId)?;
        let session_id_e = result.get_extractor(SessionExerciseIden::SessionId)?;
        let creation_date_e = result.get_extractor(SessionExerciseIden::CreationDate)?;
        let last_updated_date_e = result.get_extractor(SessionExerciseIden::LastUpdatedDate)?;

        (0..result.result_rows.len())
            .into_iter()
            .map(|i| {
                let res = SessionExercise {
                    id: id_e(&result, i).and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                    exercise_id: exercise_id_e(&result, i)?,
                    session_id: session_id_e(&result, i)?,
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
