//! Shared code that interacts with the browser in ways the service worker and
//! client both are likely to want to do

mod record_subscription;
pub use record_subscription::*;
