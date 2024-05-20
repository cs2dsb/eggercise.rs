use serde::{Deserialize, Serialize};
use crate::{ model::User, types::Uuid };

#[cfg(feature="backend")]
use rusqlite::Connection;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserId {
    pub id: Uuid,
}

impl From<&User> for UserId {
    fn from(value: &User) -> Self {
        Self {
            id: value.id.clone()
        }
    }
}

#[cfg(feature="backend")]
impl UserId {
    pub fn fetch_full_user(&self, conn: &Connection) -> Result<User, anyhow::Error> {
        let user = User::fetch_by_id(conn, &self.id)?;
        Ok(user)
    }
}