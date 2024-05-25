use axum::{ Json, http::StatusCode};

pub async fn ping() -> (StatusCode, Json<()>) {
    (StatusCode::OK, Json(()))
}
