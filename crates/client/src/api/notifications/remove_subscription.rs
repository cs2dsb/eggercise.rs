use gloo_net::http::Method;
use shared::api::{
    self,
    error::{FrontendError, ServerError},
    response_errors::FetchError,
};

use crate::api::json_request;

pub async fn remove_subscription() -> Result<(), FrontendError<ServerError<FetchError>>> {
    json_request::<_, (), _>(
        Method::DELETE,
        api::Object::PushSubscription.path(),
        None::<&()>,
    )
    .await
}
