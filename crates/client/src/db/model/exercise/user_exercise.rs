use shared::{
    model::{UserExercise, UserExerciseIden},
    types::Uuid,
};

use crate::{
    db::PromiserFetcher,
    utils::sqlite3::{parse_datetime, ExecResult, SqlitePromiserError},
};

impl PromiserFetcher for UserExercise {
    fn extract_fields(result: ExecResult) -> Result<Vec<Self>, SqlitePromiserError> {
        let id_e = result.get_extractor(UserExerciseIden::Id)?;
        let exercise_id_e = result.get_extractor(UserExerciseIden::ExerciseId)?;
        let user_id_e = result.get_extractor(UserExerciseIden::UserId)?;
        let recovery_days_e = result.get_extractor(UserExerciseIden::RecoveryDays)?;
        let creation_date_e = result.get_extractor(UserExerciseIden::CreationDate)?;
        let last_updated_date_e = result.get_extractor(UserExerciseIden::LastUpdatedDate)?;

        (0..result.result_rows.len())
            .into_iter()
            .map(|i| {
                let res = UserExercise {
                    id: id_e(&result, i).and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                    exercise_id: exercise_id_e(&result, i)?,
                    user_id: user_id_e(&result, i)?,
                    recovery_days: recovery_days_e(&result, i)?,
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
