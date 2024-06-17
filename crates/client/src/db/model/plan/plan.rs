use shared::{
    model::{Plan, PlanIden},
    types::Uuid,
};

use crate::{
    db::PromiserFetcher,
    utils::sqlite3::{parse_datetime, ExecResult, SqlitePromiserError},
};

impl PromiserFetcher for Plan {
    fn extract_fields(result: ExecResult) -> Result<Vec<Self>, SqlitePromiserError> {
        let id_e = result.get_extractor(PlanIden::Id)?;
        let owner_id_e = result.get_extractor(PlanIden::OwnerId)?;
        let name_e = result.get_extractor(PlanIden::Name)?;
        let description_e = result.get_extractor(PlanIden::Description)?;
        let creation_date_e = result.get_extractor(PlanIden::CreationDate)?;
        let last_updated_date_e = result.get_extractor(PlanIden::LastUpdatedDate)?;

        (0..result.result_rows.len())
            .into_iter()
            .map(|i| {
                let res = Plan {
                    id: id_e(&result, i).and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                    owner_id: owner_id_e(&result, i)?,
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
