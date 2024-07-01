use gloo_net::http::Method;

use crate::{
    api::{
        self,
        error::{FrontendError, ServerError},
        payloads::VapidResponse,
        response_errors::FetchError,
    },
    utils::fetch::json_request,
};

pub async fn fetch_vapid() -> Result<VapidResponse, FrontendError<ServerError<FetchError>>> {
    json_request::<_, VapidResponse, _>(Method::GET, api::Object::Vapid.path(), None::<&()>).await
}
