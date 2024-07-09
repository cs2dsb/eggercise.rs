use const_format::concatcp;

use crate::api::API_BASE_PATH;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Auth {
    RegisterStart,
    RegisterFinish,
    LoginStart,
    LoginFinish,
    RegisterNewKeyStart,
    RegisterNewKeyFinish,
    CreateTemporaryLogin,
    TemporaryLogin,
}

impl Auth {
    pub const fn path(&self) -> &str {
        use Auth::*;
        match self {
            RegisterStart => concatcp!(API_BASE_PATH, "auth/register/start"),
            RegisterFinish => concatcp!(API_BASE_PATH, "auth/register/finish"),
            LoginStart => concatcp!(API_BASE_PATH, "auth/login/start"),
            LoginFinish => concatcp!(API_BASE_PATH, "auth/login/finish"),
            RegisterNewKeyStart => concatcp!(API_BASE_PATH, "auth/register_key/start"),
            RegisterNewKeyFinish => concatcp!(API_BASE_PATH, "auth/register_key/finish"),
            CreateTemporaryLogin => concatcp!(API_BASE_PATH, "auth/temporary_login/create"),
            TemporaryLogin => concatcp!(API_BASE_PATH, "auth/login/code/:id"),
        }
    }
}
