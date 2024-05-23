use gloo_net::http::Method;
use shared::{api::{
    self,
    error::{FrontendError, ServerError},
    response_errors::TemporaryLoginError,
}, model::TemporaryLogin};

use super::json_request;

pub async fn create_temporary_login() -> Result<TemporaryLogin, FrontendError<ServerError<TemporaryLoginError>>> {
    Ok(json_request(
        Method::POST, 
        api::Auth::CreateTemporaryLogin.path(), 
        None::<&()>,
    )
    .await?)
}
