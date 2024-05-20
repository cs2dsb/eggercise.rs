use gloo_net::http::Method;

use shared::{
    api::{self, error::FrontendError, response_errors::FetchError},
    model::User,
};
use super::json_request;

pub async fn fetch_user() -> Result<User, FrontendError<FetchError>> {
    json_request::<_, User, _>(
        Method::GET, 
        api::Object::User.path(),
        None::<&()>)
        .await
}