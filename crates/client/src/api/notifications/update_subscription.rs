use gloo_net::http::Method;
use shared::{
    api::{
        self,
        error::{FrontendError, NoValidation, ServerError},
        payloads::{UpdateSubscriptionRequest, UpdateSubscriptionResponse},
        response_errors::FetchError,
    },
    model::PushNotificationSubscription,
    utils::get_subscription_key,
};
use web_sys::{PushEncryptionKeyName, PushSubscription};

use crate::api::json_request;

pub async fn update_subscription(
    subscription: &PushSubscription,
) -> Result<UpdateSubscriptionResponse, FrontendError<ServerError<FetchError>>> {
    let endpoint = subscription.endpoint();
    let key = get_subscription_key(&subscription, PushEncryptionKeyName::P256dh)?;
    let auth = get_subscription_key(&subscription, PushEncryptionKeyName::Auth)?;

    let subscription = PushNotificationSubscription {
        endpoint,
        key,
        auth,
    };

    json_request::<_, UpdateSubscriptionResponse, _>(
        Method::POST,
        api::Object::PushSubscription.path(),
        Some(&NoValidation(UpdateSubscriptionRequest {
            subscription,
        })),
    )
    .await
}
