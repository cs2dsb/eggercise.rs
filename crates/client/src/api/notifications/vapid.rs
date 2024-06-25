use gloo_net::http::Method;
use shared::api::{
    self,
    error::{FrontendError, ServerError},
    payloads::VapidResponse,
    response_errors::FetchError,
};

use crate::api::json_request;

pub async fn fetch_vapid() -> Result<VapidResponse, FrontendError<ServerError<FetchError>>> {
    json_request::<_, VapidResponse, _>(Method::GET, api::Object::Vapid.path(), None::<&()>).await
}
