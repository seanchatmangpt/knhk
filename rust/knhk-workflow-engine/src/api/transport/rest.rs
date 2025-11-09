//! REST transport adapter
//!
//! Converts between REST-specific types (axum::Json, StatusCode) and unified models.

use crate::api::models::errors::ApiError;
use axum::{http::StatusCode, response::IntoResponse, Json as AxumJson};

/// REST adapter for converting between REST types and unified models
pub struct RestAdapter;

impl RestAdapter {
    /// Convert ApiError to HTTP StatusCode
    pub fn error_to_status(error: &ApiError) -> StatusCode {
        error.to_http_status()
    }

    /// Convert ApiError to JSON error response
    pub fn error_to_response(error: ApiError) -> axum::response::Response {
        let status = error.to_http_status();
        let body = serde_json::json!({
            "error": {
                "code": error.code,
                "message": error.message,
                "details": error.details
            }
        });
        (status, AxumJson(body)).into_response()
    }

    /// Convert Result<T, ApiError> to REST response
    pub fn result_to_response<T: serde::Serialize>(
        result: Result<T, ApiError>,
    ) -> axum::response::Response {
        match result {
            Ok(data) => (StatusCode::OK, AxumJson(data)).into_response(),
            Err(error) => Self::error_to_response(error).into_response(),
        }
    }
}
