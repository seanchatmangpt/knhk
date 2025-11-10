//! gRPC transport adapter
//!
//! Converts between gRPC-specific types (tonic::Request, tonic::Response, tonic::Status) and unified models.

#[cfg(feature = "grpc")]
use crate::api::models::errors::ApiError;
#[cfg(feature = "grpc")]
use tonic::Status;

#[cfg(feature = "grpc")]
/// gRPC adapter for converting between gRPC types and unified models
pub struct GrpcAdapter;

#[cfg(feature = "grpc")]
impl GrpcAdapter {
    /// Convert ApiError to gRPC Status
    pub fn error_to_status(error: ApiError) -> Status {
        error.to_grpc_status()
    }

    /// Convert Result<T, ApiError> to gRPC response
    pub fn result_to_response<T>(
        result: Result<T, ApiError>,
    ) -> Result<tonic::Response<T>, Status> {
        result
            .map_err(GrpcAdapter::error_to_status)
            .map(tonic::Response::new)
    }
}
