use const_format::concatcp;

use crate::api::API_BASE_PATH;

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
    Log,
    RtcOffer,
    RtcSignalling,
    RtcStun,
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
            Log => concatcp!(API_BASE_PATH, "log"),
            RtcOffer => concatcp!(API_BASE_PATH, "rtc/offer"),
            RtcSignalling => concatcp!(API_BASE_PATH, "rtc/signalling"),
            RtcStun => concatcp!(API_BASE_PATH, "rtc/stun"),
        }
    }
}
