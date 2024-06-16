use shared::{
    model::{User, UserIden},
    types::Uuid,
};

use crate::{
    db::PromiserFetcher,
    utils::sqlite3::{parse_datetime, ExecResult, SqlitePromiserError},
};

impl PromiserFetcher for User {
    fn extract_fields(result: ExecResult) -> Result<Vec<Self>, SqlitePromiserError> {
        let id_e = result.get_extractor(UserIden::Id)?;
        let username_e = result.get_extractor(UserIden::Username)?;
        let email_e = result.get_extractor(UserIden::Email)?;
        let display_name_e = result.get_extractor(UserIden::DisplayName)?;
        let registration_date_e = result.get_extractor(UserIden::RegistrationDate)?;
        let last_updated_date_e = result.get_extractor(UserIden::LastUpdatedDate)?;
        let last_login_date_e = result.get_extractor(UserIden::LastLoginDate)?;

        (0..result.result_rows.len())
            .into_iter()
            .map(|i| {
                let res = User {
                    id: id_e(&result, i).and_then(|s: String| Ok(Uuid::parse(&s)?))?,
                    username: username_e(&result, i)?,
                    email: email_e(&result, i)?,
                    display_name: display_name_e(&result, i)?,
                    registration_date: registration_date_e(&result, i)
                        .and_then(|s: String| Ok(parse_datetime(&s)?))?,
                    last_updated_date: last_updated_date_e(&result, i)
                        .and_then(|s: String| Ok(parse_datetime(&s)?))?,
                    last_login_date: last_login_date_e(&result, i).and_then(
                        |s: Option<String>| s.map(|s| Ok(parse_datetime(&s)?)).transpose(),
                    )?,
                };

                Ok::<_, SqlitePromiserError>(res)
            })
            .collect::<Result<Vec<_>, _>>()
    }
}
