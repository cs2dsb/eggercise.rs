use serde::{Deserialize, Serialize};

use crate::model::PushNotificationSubscription;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubscriptionRequest {
    pub subscription: PushNotificationSubscription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubscriptionResponse {}
