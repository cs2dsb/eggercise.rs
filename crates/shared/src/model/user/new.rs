use serde::{Deserialize, Serialize};
#[cfg(feature = "backend")]
use {
    crate::{
        api::error::{ServerError, ServerErrorContext},
        model::{Credential, NewCredential, User},
    },
    exemplar::Model,
    rusqlite::Connection,
    std::error::Error,
    webauthn_rs::prelude::Passkey,
};

use crate::types::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "backend", derive(Model))]
#[cfg_attr(feature = "backend", table("user"))]
pub struct NewUser {
    pub id: Uuid,
    pub username: String,
}

impl NewUser {
    pub fn new<I: Into<Uuid>, T: Into<String>>(id: I, username: T) -> Self {
        Self {
            id: id.into(),
            username: username.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg(feature = "backend")]
pub struct NewUserWithPasskey {
    pub id: Uuid,
    pub username: String,
    pub passkey: Passkey,
}

#[cfg(feature = "backend")]
impl NewUserWithPasskey {
    fn split(self) -> (NewUser, Passkey) {
        let Self {
            id,
            username,
            passkey,
        } = self;
        (NewUser::new(id, username), passkey)
    }
    pub fn new<I: Into<Uuid>, T: Into<String>>(id: I, username: T, passkey: Passkey) -> Self {
        Self {
            id: id.into(),
            username: username.into(),
            passkey,
        }
    }

    pub fn create<T: Error>(
        self,
        conn: &mut Connection,
    ) -> Result<(User, Credential), ServerError<T>> {
        let tx = conn.transaction()?;

        let (new_user, passkey) = self.split();
        let user_id = new_user.id.clone();
        let new_credential = NewCredential::new(new_user.id.clone(), passkey.into());

        let user = {
            new_user
                .insert(&tx)
                .context("NewUserWithPasskey::insert(User)")?;

            User::fetch_by_id(&tx, &user_id).context("NewUserWithPasskey::fetch(User)")?
        };

        let credential = {
            new_credential
                .insert(&tx)
                .context("NewUserWithPasskey::insert(Credential)")?;
            Credential::fetch(&tx, &new_credential.id)
                .context("NewUserWithPasskey::fetch(Credential)")?
        };

        tx.commit()?;

        Ok((user, credential))
    }
}
