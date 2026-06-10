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

use http::StatusCode;
use serde::Serialize;
use serde_json::{json, Value};

// ── Structured error detail ───────────────────────────────────────────────────

/// Field-level validation detail for structured error responses.
///
/// Used by APIs that return per-field validation failures alongside a top-level
/// error message. `field` is `None` for non-field errors (e.g. business rule
/// violations).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorDetail {
    pub field: Option<String>,
    pub message: String,
}

// ── Simple error type ─────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    Forbidden,
    /// A short message describing the bad input. Sent verbatim in the response.
    BadRequest(String),
    /// A short message describing the conflicting state (e.g. duplicate
    /// resource). Sent verbatim in the response with a 409 status.
    Conflict(String),
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

impl ApiError {
    /// Render this error into its HTTP status code and JSON body, independent of
    /// any web framework. Server-side variants (`Database`, `Internal`) are
    /// logged here so callers don't have to.
    ///
    /// This is the single source of truth for the response contract; the
    /// feature-gated `IntoResponse` impls delegate to it.
    pub fn status_and_body(self) -> (StatusCode, Value) {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, json!({ "error": "not_found" })),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, json!({ "error": "forbidden" })),
            ApiError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                json!({ "error": "bad_request", "message": msg }),
            ),
            ApiError::Conflict(msg) => (
                StatusCode::CONFLICT,
                json!({ "error": "conflict", "message": msg }),
            ),
            ApiError::Database(e) => {
                tracing::error!(error = ?e, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "internal_error" }),
                )
            }
            ApiError::Internal(e) => {
                tracing::error!(error = ?e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "internal_error" }),
                )
            }
        }
    }
}

// ── axum 0.8 integration ──────────────────────────────────────────────

#[cfg(feature = "axum-08")]
impl axum_08::response::IntoResponse for ApiError {
    fn into_response(self) -> axum_08::response::Response {
        let (status, body) = self.status_and_body();
        (status, axum_08::Json(body)).into_response()
    }
}

// ── axum 0.7 integration ──────────────────────────────────────────────

#[cfg(feature = "axum-07")]
impl axum_07::response::IntoResponse for ApiError {
    fn into_response(self) -> axum_07::response::Response {
        let (status, body) = self.status_and_body();
        (status, axum_07::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_shape() {
        let (status, body) = ApiError::NotFound.status_and_body();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body, json!({ "error": "not_found" }));
    }

    #[test]
    fn bad_request_carries_message() {
        let (status, body) = ApiError::BadRequest("bad id".into()).status_and_body();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body, json!({ "error": "bad_request", "message": "bad id" }));
    }

    #[test]
    fn conflict_carries_message() {
        let (status, body) = ApiError::Conflict("already exists".into()).status_and_body();
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(
            body,
            json!({ "error": "conflict", "message": "already exists" })
        );
    }

    #[test]
    fn internal_is_opaque() {
        let (status, body) = ApiError::Internal(anyhow::anyhow!("boom")).status_and_body();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body, json!({ "error": "internal_error" }));
    }
}
