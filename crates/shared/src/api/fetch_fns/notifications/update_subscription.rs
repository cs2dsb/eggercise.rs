use gloo::net::http::Method;
use web_sys::{PushEncryptionKeyName, PushSubscription};

use crate::{
    api::{
        self,
        error::{FrontendError, NoValidation, ServerError},
        payloads::{UpdateSubscriptionRequest, UpdateSubscriptionResponse},
        response_errors::FetchError,
    },
    model::PushNotificationSubscription,
    utils::{fetch::json_request, get_subscription_key},
};

pub async fn update_subscription(
    subscription: &PushSubscription,
) -> Result<UpdateSubscriptionResponse, FrontendError<ServerError<FetchError>>> {
    let endpoint = subscription.endpoint();
    let key = get_subscription_key(&subscription, PushEncryptionKeyName::P256dh)?;
    let auth = get_subscription_key(&subscription, PushEncryptionKeyName::Auth)?;

    let subscription = PushNotificationSubscription { endpoint, key, auth };

    json_request::<_, UpdateSubscriptionResponse, _>(
        Method::POST,
        api::Object::PushSubscription.path(),
        Some(&NoValidation(UpdateSubscriptionRequest { subscription })),
    )
    .await
}
