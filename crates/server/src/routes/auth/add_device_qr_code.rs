use axum::{ 
    http::{ header, HeaderMap, HeaderValue }, 
    response::IntoResponse, 
};

use shared::api::error::{Nothing, ServerError};
use qrcodegen::{ QrCode, QrCodeEcc, QrSegment, Version };
use webauthn_rs::prelude::Uuid;

use crate::{db::DatabaseConnection, SessionValue, UserState};

fn to_svg_string(qr: &QrCode, border: u16) -> String {
    let border = border as i32;
	let dimension = qr.size() + border * 2;

	let mut result = String::new();
	result += "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
	result += "<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">\n";
	result += &format!("<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" viewBox=\"0 0 {0} {0}\" stroke=\"none\">\n", dimension);
	result += "\t<rect width=\"100%\" height=\"100%\" fill=\"#FFFFFF\"/>\n";
	result += "\t<path d=\"";
	
    for y in 0 .. qr.size() {
		for x in 0 .. qr.size() {
			if qr.get_module(x, y) {
				if x != 0 || y != 0 {
					result += " ";
				}
				result += &format!("M{},{}h1v1h-1z", x + border, y + border);
			}
		}
	}
	
    result += "\" fill=\"#000000\"/>\n";
	result += "</svg>\n";
	result
}

pub async fn add_device_qr_code(
    DatabaseConnection(_conn): DatabaseConnection,
    _session: SessionValue,
    user_state: UserState,
) -> Result<impl IntoResponse, ServerError<Nothing>> {
    let data = format!("https://egg.ileet.co.uk/user/{}/add_device/{}", 
        user_state.id.id.as_hyphenated(),
        Uuid::new_v4().as_hyphenated(),
    );

    let segments = QrSegment::make_segments(&data);
    let code = QrCode::encode_segments_advanced(
        &segments, 
        QrCodeEcc::Medium, 
        Version::MIN, 
        Version::MAX,
        None, 
        true,
    ).map_err(|e| ServerError::Other { message: format!("QrCode error: {:?}", e) })?;
    
    let image = to_svg_string(&code, 3);

    let headers = {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_str(
            mime::IMAGE_SVG.essence_str())
            .map_err(|e| ServerError::Other { message: format!("Parsing header value failed: {:?}", e) })?);
        headers
    };

    Ok((headers, image))
}
