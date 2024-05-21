use const_format::concatcp;
pub mod response_errors;
pub mod error;

pub const API_BASE_PATH: &str = "/api/";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Object {
    User,
}

impl Object {
    pub const fn path(&self) -> &str {
        use Object::*;
        match self {
            User => concatcp!(API_BASE_PATH, "user"),
        }
    }
    
    pub const fn id_path(&self) -> &str {
        use Object::*;
        match self {
            User => concatcp!(API_BASE_PATH, "user/:id"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Auth {
    RegisterStart,
    RegisterFinish,
    LoginStart,
    LoginFinish,
    RegisterNewKeyStart,
    RegisterNewKeyFinish,
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
        }
    }
}