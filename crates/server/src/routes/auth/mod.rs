//! Auth is based on https://github.com/kanidm/webauthn-rs/blob/628599aa47b5c120e7f29cce8c526af532fba9ce/tutorial/server/axum/src/auth.rs#L52
//!
//! Currently register, login and add key are implemented. Add key is somewhat
//! useless for the most obvious case of wanting to grant your phone access to
//! an account registered on desktop or vice versa. All that can currently be
//! done is add multiple devices from the same browser.
//!
//! What is really needed to add multi-device access is a process similar to how
//! google/facebook/etc does it where they pop up an alert on an existing device
//! to grant access to the new device. Not sure how to accomplish this.

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