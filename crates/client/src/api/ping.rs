use gloo_net::http::Method;
use shared::api::{
    self,
    error::{FrontendError, Nothing, ServerError},
};

use super::json_request;

pub async fn ping() -> Result<(), FrontendError<ServerError<Nothing>>> {
    json_request::<_, (), _>(Method::GET, api::Object::Ping.path(), None::<&()>).await
}
