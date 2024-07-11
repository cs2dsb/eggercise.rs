use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::{feature_model_derives, feature_model_imports, types::Uuid};

feature_model_imports!();

#[cfg(feature = "backend")]
use {
    crate::{api::error::ServerError, model::Model},
    rusqlite::OptionalExtension,
    sea_query::Order,
    semver::Version,
    std::error::Error,
};

feature_model_derives!(
    "service_version",
    "../../migrations/012-service_version/up.sql",
    pub struct ServiceVersion {
        pub id: Uuid,
        pub version: String,
        pub creation_date: DateTime<Utc>,
    }
);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "backend", derive(ExemplarModel))]
#[cfg_attr(feature = "backend", table("service_version"))]
pub struct NewServiceVersion {
    pub id: Uuid,
    pub version: String,
}

#[cfg(feature = "backend")]
impl NewServiceVersion {
    pub fn new(version: String) -> Result<Self, semver::Error> {
        // Just to check it's valid semver
        let _ = Version::parse(&version)?;
        let id = Uuid::new_v4();
        Ok(Self { id, version })
    }
}

impl Display for ServiceVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.version.fmt(f)
    }
}
#[cfg(feature = "backend")]
impl ServiceVersion {
    pub fn cmp(&self, other: &str) -> Result<std::cmp::Ordering, semver::Error> {
        let my_version = Version::parse(&self.version)?;
        let other_version = Version::parse(other)?;

        Ok(my_version.cmp(&other_version))
    }

    pub fn fetch_latest<T: Error>(conn: &Connection) -> Result<Option<Self>, ServerError<T>> {
        let (sql, values) = Self::select_star()
            .order_by(<Self as Model>::Iden::CreationDate, Order::Desc)
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let value =
            stmt.query_row(&*values.as_params(), <Self as ExemplarModel>::from_row).optional()?;

        Ok(value)
    }

    pub fn create<T: Error>(
        conn: &mut Connection,
        new_service_version: NewServiceVersion,
    ) -> Result<ServiceVersion, ServerError<T>> {
        let tx = conn.transaction()?;
        let new_service_version = {
            new_service_version.insert(&tx)?;
            ServiceVersion::fetch_by_id(&tx, &new_service_version.id)?
        };
        tx.commit()?;

        Ok(new_service_version)
    }
}
