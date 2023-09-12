use axum::{response::IntoResponse, response::Response};
use http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum ApiError {
    AuthenticationRequired,
    AuthenticationExpired,
    Unauthorized,
    TokenGenerationError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status: StatusCode = match self {
            ApiError::AuthenticationRequired => StatusCode::UNAUTHORIZED,
            ApiError::AuthenticationExpired => StatusCode::UNAUTHORIZED,
            ApiError::Unauthorized => StatusCode::FORBIDDEN,
            ApiError::TokenGenerationError => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let payload = postcard::to_stdvec(&self).unwrap();
        (status, payload).into_response()
    }
}