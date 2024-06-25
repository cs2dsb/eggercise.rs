use const_format::concatcp;
pub mod error;
pub mod payloads;
pub mod response_errors;

pub const API_BASE_PATH: &str = "/api/";
pub const CSRF_HEADER: &str = "X-CSRF-TOKEN";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Object {
    User,
    UserId,
    QrCodeId,
    Ping,
    Websocket,
    PushNotification,
    PushSubscription,
    Vapid,
}

impl Object {
    pub const fn path(&self) -> &str {
        use Object::*;
        match self {
            User => concatcp!(API_BASE_PATH, "user"),
            QrCodeId => concatcp!(API_BASE_PATH, "qrcode/:id"),
            UserId => concatcp!(API_BASE_PATH, "user/:id"),
            Ping => concatcp!(API_BASE_PATH, "ping"),
            Websocket => concatcp!(API_BASE_PATH, "ws"),
            PushNotification => concatcp!(API_BASE_PATH, "push_notification"),
            PushSubscription => concatcp!(API_BASE_PATH, "push_subscription"),
            Vapid => concatcp!(API_BASE_PATH, "vapid"),
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
