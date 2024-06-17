use gloo_utils::format::JsValueSerdeExt;
use shared::{
    model::{PlanExerciseGroup, PlanExerciseGroupIden},
    types::Uuid,
};
use wasm_bindgen::JsValue;

use crate::{
    db::PromiserFetcher,
    utils::sqlite3::{parse_datetime, ExecResult, SqlitePromiserError},
};

impl PromiserFetcher for PlanExerciseGroup {
    fn extract_fields(result: ExecResult) -> Result<Vec<Self>, SqlitePromiserError> {
        let id_e = result.get_extractor(PlanExerciseGroupIden::Id)?;
        let plan_id_e = result.get_extractor(PlanExerciseGroupIden::PlanId)?;
        let exercise_group_id = result.get_extractor(PlanExerciseGroupIden::ExerciseGroupId)?;
        let notes_e = result.get_extractor(PlanExerciseGroupIden::Notes)?;
        let config_e = result.get_extractor(PlanExerciseGroupIden::Config)?;
        let creation_date_e = result.get_extractor(PlanExerciseGroupIden::CreationDate)?;
        let last_updated_date_e = result.get_extractor(PlanExerciseGroupIden::LastUpdatedDate)?;

        (0..result.result_rows.len())
            .into_iter()
            .map(|i| {
                let res = PlanExerciseGroup {
                    id: id_e(&result, i).and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                    plan_id: plan_id_e(&result, i)?,
                    exercise_group_id: exercise_group_id(&result, i)?,
                    notes: notes_e(&result, i)?,
                    config: config_e(&result, i).and_then(|s: String| {
                        Ok(JsValueSerdeExt::into_serde(&JsValue::from(s))?)
                    })?,
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
