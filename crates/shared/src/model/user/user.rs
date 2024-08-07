use chrono::{DateTime, Utc};

use crate::{feature_model_derives, feature_model_imports, types::Uuid};

feature_model_imports!();

#[cfg(feature = "backend")]
use {
    crate::{
        api::error::{ServerError, ServerErrorContext},
        model::{Credential, NewCredential, NewUser, TemporaryLogin},
    },
    std::error::Error,
    webauthn_rs::prelude::Passkey,
};

use super::PushNotificationSubscription;

feature_model_derives!(
    "user",
    "../../../migrations/001-user/up.sql",
    pub struct User {
        pub id: Uuid,
        pub username: String,
        pub email: Option<String>,
        pub display_name: Option<String>,
        pub push_notification_subscription: Option<PushNotificationSubscription>,
        pub creation_date: DateTime<Utc>,
        pub last_updated_date: DateTime<Utc>,
        pub last_login_date: Option<DateTime<Utc>>,
    }
);

#[cfg(feature = "wasm")]
impl crate::model::model_into_view::UseDefaultModelView for User {}

#[cfg(feature = "backend")]
impl User {
    pub fn fetch_by_id(conn: &Connection, id: &Uuid) -> Result<User, rusqlite::Error> {
        let (sql, values) = Self::select_star()
            .and_where(Expr::col(UserIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let user = stmt.query_row(&*values.as_params(), User::from_row)?;
        Ok(user)
    }

    pub fn create<T: Error>(
        conn: &mut Connection,
        new_user: NewUser,
    ) -> Result<User, ServerError<T>> {
        let tx = conn.transaction()?;
        let user = {
            new_user.insert(&tx)?;
            User::fetch_by_id(&tx, &new_user.id)?
        };
        tx.commit()?;

        Ok(user)
    }

    pub fn update<T: Error>(&self, conn: &Connection) -> Result<(), ServerError<T>> {
        let (sql, values) = Query::update()
            .table(UserIden::Table)
            .values([
                (UserIden::Username, self.username.clone().into()),
                (UserIden::Email, self.email.clone().into()),
                (UserIden::DisplayName, self.display_name.clone().into()),
                (
                    UserIden::PushNotificationSubscription,
                    self.push_notification_subscription.clone().into(),
                ),
                (UserIden::CreationDate, self.creation_date.into()),
                (UserIden::LastUpdatedDate, self.last_updated_date.into()),
                (UserIden::LastLoginDate, self.last_login_date.into()),
            ])
            .and_where(Expr::col(UserIden::Id).eq(&self.id))
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        stmt.execute(&*values.as_params())?;

        Ok(())
    }

    pub fn add_passkey<T: Error>(
        mut self,
        conn: &mut Connection,
        passkey: Passkey,
    ) -> Result<Credential, ServerError<T>> {
        let tx = conn.transaction()?;

        let new_credential = NewCredential::new(self.id.clone(), passkey.into());

        let credential = {
            new_credential.insert(&tx).context("User::add_passkey::insert(Credential)")?;
            Credential::fetch(&tx, &new_credential.id)
                .context("User::add_passkey::fetch(Credential)")?
        };

        self.last_updated_date = credential.creation_date;
        self.update(&tx)?;

        tx.commit()?;

        Ok(credential)
    }

    pub fn temporary_login<T: Error>(
        &self,
        conn: &Connection,
    ) -> Result<Option<TemporaryLogin>, ServerError<T>> {
        TemporaryLogin::fetch_by_user_id(conn, &self.id)
    }

    pub fn fetch_all_with_push_notifications_enabled(
        conn: &Connection,
    ) -> Result<Vec<User>, rusqlite::Error> {
        let (sql, values) = Self::select_star()
            .and_where(Expr::col(UserIden::PushNotificationSubscription).is_not_null())
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;

        let results = stmt
            .query_and_then(&*values.as_params(), Self::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }
}
