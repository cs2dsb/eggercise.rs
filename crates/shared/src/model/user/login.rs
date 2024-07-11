use serde::{Deserialize, Serialize};

use crate::{
    api::error::ValidationError,
    model::{constants::USERNAME_MIN_LENGTH, ValidateModel},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginUser {
    pub username: String,
}

impl LoginUser {
    pub fn new<T: Into<String>>(username: T) -> Self {
        let username = username.into();
        Self { username }
    }
}

impl ValidateModel for LoginUser {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.username.len() < USERNAME_MIN_LENGTH {
            Err(ValidationError {
                error_messages: vec![format!(
                    "Username needs to be at least {USERNAME_MIN_LENGTH} characters long"
                )],
            })
        } else {
            Ok(())
        }
    }
}
