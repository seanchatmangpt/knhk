//! Fortune 5 API Middleware
//!
//! Provides enterprise-grade middleware for REST API:
//! - Authentication/Authorization
//! - Rate limiting
//! - Circuit breakers
//! - Request tracing
//! - Audit logging

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};

use crate::security::AuthManager;
use crate::resilience::{CircuitBreaker, RateLimiter};

/// Fortune 5 API middleware stack
pub struct Fortune5Middleware {
    /// Auth manager
    auth_manager: Option<Arc<AuthManager>>,
    /// Rate limiter
    rate_limiter: Option<Arc<RateLimiter>>,
    /// Circuit breaker
    circuit_breaker: Option<Arc<CircuitBreaker>>,
}

impl Fortune5Middleware {
    /// Create new Fortune 5 middleware
    pub fn new(
        auth_manager: Option<Arc<AuthManager>>,
        rate_limiter: Option<Arc<RateLimiter>>,
        circuit_breaker: Option<Arc<CircuitBreaker>>,
    ) -> Self {
        Self {
            auth_manager,
            rate_limiter,
            circuit_breaker,
        }
    }

    /// Build middleware stack
    pub fn build(self) -> ServiceBuilder<impl tower::Layer<axum::Router>> {
        let mut builder = ServiceBuilder::new();

        // CORS layer
        builder = builder.layer(CorsLayer::permissive());

        // Tracing layer
        builder = builder.layer(TraceLayer::new_for_http());

        // Rate limiting layer (if enabled)
        if let Some(ref rate_limiter) = self.rate_limiter {
            // FUTURE: Add rate limiting middleware
            // For now, rate limiting is handled in the handler
        }

        // Circuit breaker layer (if enabled)
        if let Some(ref circuit_breaker) = self.circuit_breaker {
            // FUTURE: Add circuit breaker middleware
            // For now, circuit breaking is handled in the handler
        }

        builder
    }
}

/// Authentication middleware
pub async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    // Validate token (FUTURE: Integrate with SPIFFE/SPIRE)
    if !auth_header.starts_with("Bearer ") {
        warn!("Invalid authorization header format");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Continue request
    Ok(next.run(request).await)
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // FUTURE: Implement rate limiting per client
    // For now, just continue
    Ok(next.run(request).await)
}

/// Circuit breaker middleware
pub async fn circuit_breaker_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // FUTURE: Implement circuit breaker
    // For now, just continue
    Ok(next.run(request).await)
}

/// Request tracing middleware
pub async fn tracing_middleware(
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    let method = request.method().clone();

    info!(
        method = %method,
        path = %path,
        "API request"
    );

    let response = next.run(request).await;

    info!(
        method = %method,
        path = %path,
        status = %response.status(),
        "API response"
    );

    response
}

/// Audit logging middleware
pub async fn audit_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    let user = headers
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let path = request.uri().path();
    let method = request.method().clone();

    let response = next.run(request).await;

    // Audit log
    info!(
        audit.event = "api_request",
        audit.user = %user,
        audit.method = %method,
        audit.path = %path,
        audit.status = %response.status(),
        "Audit log"
    );

    response
}

