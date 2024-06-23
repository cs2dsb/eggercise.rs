use sea_query::SqliteQueryBuilder;
use shared::{
    model::{Model, Session, SessionIden},
    types::Uuid,
};

use crate::db::{
    sqlite3::{parse_datetime, ExecResult, SqlitePromiserError},
    PromiserFetcher, PromiserInserter,
};

impl PromiserFetcher for Session {
    fn extract_fields(result: ExecResult) -> Result<Vec<Self>, SqlitePromiserError> {
        let id_e = result.get_extractor(SessionIden::Id)?;
        let plan_instance_id_e = result.get_extractor(SessionIden::PlanInstanceId)?;
        let planned_date_e = result.get_extractor(SessionIden::PlannedDate)?;
        let performed_date_e = result.get_extractor(SessionIden::PerformedDate)?;
        let creation_date_e = result.get_extractor(SessionIden::CreationDate)?;
        let last_updated_date_e = result.get_extractor(SessionIden::LastUpdatedDate)?;

        (0..result.result_rows.len())
            .into_iter()
            .map(|i| {
                let res = Session {
                    id: id_e(&result, i).and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                    plan_instance_id: plan_instance_id_e(&result, i)
                        .and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                    planned_date: planned_date_e(&result, i)
                        .and_then(|s: String| Ok(parse_datetime(&s)?))?,
                    performed_date: performed_date_e(&result, i).and_then(
                        |s: Option<String>| s.map(|s| Ok(parse_datetime(&s)?)).transpose(),
                    )?,
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

impl PromiserInserter for Session {
    fn insert_sql(&self) -> Result<String, SqlitePromiserError> {
        Ok(Self::insert_query()
            .values([
                (&self.id).into(),
                (&self.plan_instance_id).into(),
                sea_query::Value::ChronoDateTimeUtc(Some(Box::new(self.planned_date.clone())))
                    .into(),
                sea_query::Value::ChronoDateTimeUtc(
                    self.performed_date.as_ref().map(|d| Box::new(d.clone())),
                )
                .into(),
                sea_query::Value::ChronoDateTimeUtc(Some(Box::new(self.creation_date.clone())))
                    .into(),
                sea_query::Value::ChronoDateTimeUtc(Some(Box::new(self.last_updated_date.clone())))
                    .into(),
            ])?
            .to_string(SqliteQueryBuilder))
    }
}
