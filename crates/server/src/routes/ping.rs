use axum::{http::StatusCode, Json};

pub async fn ping() -> (StatusCode, Json<()>) {
    (StatusCode::OK, Json(()))
}
