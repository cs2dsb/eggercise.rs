use gloo::net::http::Method;

use crate::{
    api::{
        self,
        error::{FrontendError, ServerError},
        response_errors::FetchError,
    },
    utils::fetch::json_request,
};

pub async fn remove_subscription() -> Result<(), FrontendError<ServerError<FetchError>>> {
    json_request::<_, (), _>(
        Method::DELETE,
        api::Object::PushSubscription.path(),
        None::<&()>,
    )
    .await
}
