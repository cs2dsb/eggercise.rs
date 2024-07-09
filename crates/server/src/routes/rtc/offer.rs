use axum::Json;
use shared::api::{
    error::ServerError,
    payloads::{RtcOfferError, RtcOfferRequest, RtcOfferResponse, RtcSdpType},
};
use tracing::info;

use crate::UserState;

pub async fn offer_handler(
    // User has to be logged in and we need their ID
    _user_state: UserState,
    Json(offer): Json<RtcOfferRequest>,
) -> Result<Json<RtcOfferResponse>, ServerError<RtcOfferError>> {
    info!("Offer: {:?}", offer);

    let response = RtcOfferResponse {
        type_: RtcSdpType::Answer,
        sdp: "Hello".to_string(),
        candidate: "Blah".to_string(),
    };
    Ok(Json(response))
}
