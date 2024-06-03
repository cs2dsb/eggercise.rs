use shared::{
    model::{ExerciseGroupMember, ExerciseGroupMemberIden},
    types::Uuid,
};

use crate::{
    db::PromiserFetcher,
    utils::sqlite3::{ExecResult, SqlitePromiserError},
};

impl PromiserFetcher for ExerciseGroupMember {
    fn extract_fields(result: ExecResult) -> Result<Vec<Self>, SqlitePromiserError> {
        let id_e = result.get_extractor(ExerciseGroupMemberIden::Id)?;
        let exercise_id_e = result.get_extractor(ExerciseGroupMemberIden::ExerciseId)?;
        let group_id_e = result.get_extractor(ExerciseGroupMemberIden::GroupId)?;

        (0..result.result_rows.len())
            .into_iter()
            .map(|i| {
                let res = ExerciseGroupMember {
                    id: id_e(&result, i).and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                    exercise_id: exercise_id_e(&result, i)
                        .and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                    group_id: group_id_e(&result, i).and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                };

                Ok::<_, SqlitePromiserError>(res)
            })
            .collect::<Result<Vec<_>, _>>()
    }
}
