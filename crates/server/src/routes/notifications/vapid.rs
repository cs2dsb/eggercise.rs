use axum::{extract::State, Json};
use shared::api::{
    error::{Nothing, ServerError},
    payloads::VapidResponse,
};

use crate::{UserState, VapidPubKey};

pub async fn vapid(
    // This is only required to make this route for logged in users only
    _user_state: UserState,
    State(vapid_pub_key): State<VapidPubKey>,
) -> Result<Json<VapidResponse>, ServerError<Nothing>> {
    let vapid_response = VapidResponse {
        key: vapid_pub_key.into(),
    };
    Ok(Json(vapid_response))
}
