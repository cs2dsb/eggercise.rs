use sea_query::SqliteQueryBuilder;
use shared::{
    model::{Model, SessionExercise, SessionExerciseIden},
    types::Uuid,
};

use crate::db::{
    sqlite3::{parse_datetime, serde_stringify, ExecResult, SqlitePromiserError},
    PromiserFetcher, PromiserInserter,
};

impl PromiserFetcher for SessionExercise {
    fn extract_fields(result: ExecResult) -> Result<Vec<Self>, SqlitePromiserError> {
        let id_e = result.get_extractor(SessionExerciseIden::Id)?;
        let exercise_id_e = result.get_extractor(SessionExerciseIden::ExerciseId)?;
        let session_id_e = result.get_extractor(SessionExerciseIden::SessionId)?;
        let planned_sets_e = result.get_extractor(SessionExerciseIden::PlannedSets)?;
        let performed_sets_e = result.get_extractor(SessionExerciseIden::PerformedSets)?;
        let creation_date_e = result.get_extractor(SessionExerciseIden::CreationDate)?;
        let last_updated_date_e = result.get_extractor(SessionExerciseIden::LastUpdatedDate)?;

        (0..result.result_rows.len())
            .into_iter()
            .map(|i| {
                let res = SessionExercise {
                    id: id_e(&result, i).and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                    exercise_id: exercise_id_e(&result, i)?,
                    session_id: session_id_e(&result, i)?,
                    planned_sets: planned_sets_e(&result, i)?,
                    performed_sets: performed_sets_e(&result, i)?,
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

impl PromiserInserter for SessionExercise {
    fn insert_sql(&self) -> Result<String, SqlitePromiserError> {
        Ok(Self::insert_query()
            .values([
                (&self.id).into(),
                (&self.exercise_id).into(),
                (&self.session_id).into(),
                serde_stringify(&self.planned_sets)?.into(),
                serde_stringify(&self.performed_sets)?.into(),
                sea_query::Value::ChronoDateTimeUtc(Some(Box::new(self.creation_date.clone())))
                    .into(),
                sea_query::Value::ChronoDateTimeUtc(Some(Box::new(self.last_updated_date.clone())))
                    .into(),
            ])?
            .to_string(SqliteQueryBuilder))
    }
}
