//! Auth is based on <https://github.com/kanidm/webauthn-rs/blob/628599aa47b5c120e7f29cce8c526af532fba9ce/tutorial/server/axum/src/auth.rs#L52>

mod register_start;
pub use register_start::*;

mod register_finish;
pub use register_finish::*;

mod login_start;
pub use login_start::*;

mod login_finish;
pub use login_finish::*;

mod register_new_key_start;
pub use register_new_key_start::*;

mod register_new_key_finish;
pub use register_new_key_finish::*;

mod generate_qr_code;
pub use generate_qr_code::*;

mod user;
pub use user::*;

mod temporary_login;
pub use temporary_login::*;
