use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: String,
    pub status: u16,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Satellite not found")]
    NotFound,

    #[error("Invalid request input: {0}")]
    BadRequest(String),

    #[error("SGP4 calculation error: {0}")]
    Sgp4Error(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Sgp4Error(ref msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::InternalServerError(ref msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            AppError::Unauthorized(ref msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
        };

        let body = Json(ErrorResponse {
            error: error_message,
            status: status.as_u16(),
        });

        (status, body).into_response()
    }
}
