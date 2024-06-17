use shared::{
    model::{PlanInstance, PlanInstanceIden},
    types::Uuid,
};

use crate::{
    db::PromiserFetcher,
    utils::sqlite3::{parse_datetime, ExecResult, SqlitePromiserError},
};

impl PromiserFetcher for PlanInstance {
    fn extract_fields(result: ExecResult) -> Result<Vec<Self>, SqlitePromiserError> {
        let id_e = result.get_extractor(PlanInstanceIden::Id)?;
        let plan_id_e = result.get_extractor(PlanInstanceIden::PlanId)?;
        let user_id_e = result.get_extractor(PlanInstanceIden::UserId)?;
        let start_date_e = result.get_extractor(PlanInstanceIden::StartDate)?;
        let creation_date_e = result.get_extractor(PlanInstanceIden::CreationDate)?;
        let last_updated_date_e = result.get_extractor(PlanInstanceIden::LastUpdatedDate)?;

        (0..result.result_rows.len())
            .into_iter()
            .map(|i| {
                let res = PlanInstance {
                    id: id_e(&result, i).and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                    plan_id: plan_id_e(&result, i)?,
                    user_id: user_id_e(&result, i)?,
                    start_date: start_date_e(&result, i).and_then(|s: Option<String>| {
                        s.map(|s| Ok(parse_datetime(&s)?)).transpose()
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
