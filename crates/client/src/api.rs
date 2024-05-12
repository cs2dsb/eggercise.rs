// #[derive(Clone, Copy)]
// pub struct UnauthorizedApi {
//     url: &'static str,
// }

// #[derive(Clone)]
// pub struct AuthorizedApi {
//     url: &'static str,
//     token: (),
// }

use shared::{
    api,
    model::{NewUser, User},
};

use gloo_net::http::{Request, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// TODO: move into shared?
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("JS Error: {0:?}")]
    Fetch(#[from] gloo_net::Error),
    #[error("API Error: {0:?}")]
    Api(ApiError),
}

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Self {
        Self::Api(err)
    }
}

async fn json<T: DeserializeOwned>(response: Response) -> Result<T, Error> {
    if !response.ok() {
        Err(response.json::<ApiError>().await?)?
    }
        
    Ok(response.json().await?)
}

pub async fn register(new_user: &NewUser) -> Result<User, Error> {
    let response = Request::post(api::Object::User.path())
        .json(new_user)?
        .send().await?;

    json(response).await
}