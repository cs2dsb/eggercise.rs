use std::fmt;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct AppError {
    pub code: StatusCode,
    pub message: String,
}

impl AppError {
    /// Return a plain text response error message
    pub fn new(code: StatusCode, message: String) -> Self {
        AppError {
            code,
            message,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AppError {}: {}", self.code, self.message)
    }
}

// Render AppError into a response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.code, self.message).into_response()
    }
}

// This enables using `?` on functions that return `Result<_, Error>` to turn
// them into `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<Box<dyn std::error::Error>>,
{
    #[track_caller]
    fn from(err: E) -> Self {
        let error_message = err.into().to_string();
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", error_message),
        )
    }
}
