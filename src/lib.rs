//! Standard Axum API error type with consistent JSON responses.
//!
//! Provides a generic [`ApiError`] covering the common HTTP error cases.
//! For app-specific variants (e.g. domain validation failures), define your
//! own error enum that wraps or converts from `ApiError`.
//!
//! ## Usage
//!
//! ```ignore
//! use aincrad_api_error::ApiError;
//!
//! async fn get_item(id: Uuid) -> Result<Json<Item>, ApiError> {
//!     let item = db.find(id).await?;   // sqlx::Error → ApiError::Database
//!     item.ok_or(ApiError::NotFound)
//! }
//! ```

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    Forbidden,
    /// A short message describing the bad input. Sent verbatim in the response.
    BadRequest(String),
    Database(sqlx::Error),
    Internal(anyhow::Error),
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError::Database(e)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        ApiError::Internal(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "not_found" })),
            )
                .into_response(),

            ApiError::Forbidden => (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "forbidden" })),
            )
                .into_response(),

            ApiError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "bad_request", "message": msg })),
            )
                .into_response(),

            ApiError::Database(e) => {
                tracing::error!(error = ?e, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "internal_error" })),
                )
                    .into_response()
            }

            ApiError::Internal(e) => {
                tracing::error!(error = ?e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "internal_error" })),
                )
                    .into_response()
            }
        }
    }
}
