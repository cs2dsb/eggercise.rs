use std::{fmt, ops::Deref};

#[cfg(feature = "backend")]
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{model::User, types::Uuid};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserId {
    pub id: Uuid,
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id.as_hyphenated())
    }
}
impl Deref for UserId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl From<&User> for UserId {
    fn from(value: &User) -> Self {
        Self {
            id: value.id.clone(),
        }
    }
}

#[cfg(feature = "backend")]
impl UserId {
    pub fn fetch_full_user(&self, conn: &Connection) -> Result<User, rusqlite::Error> {
        let user = User::fetch_by_id(conn, &self.id)?;
        Ok(user)
    }
}
