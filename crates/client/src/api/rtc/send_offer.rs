use gloo::net::http::Method;
use shared::{
    api::{
        self,
        error::{FrontendError, NoValidation, ServerError},
        payloads::{RtcOfferError, RtcOfferRequest, RtcOfferResponse},
    },
    utils::fetch::json_request,
};

pub async fn send_offer(
    sdp: String,
) -> Result<RtcOfferResponse, FrontendError<ServerError<RtcOfferError>>> {
    let payload = RtcOfferRequest { sdp };

    Ok(json_request(Method::POST, api::Object::RtcOffer.path(), Some(&NoValidation(payload)))
        .await?)
}
