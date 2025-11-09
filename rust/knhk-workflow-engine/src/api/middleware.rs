//! Fortune 5 API Middleware
//!
//! Provides enterprise-grade middleware for REST API:
//! - Authentication/Authorization
//! - Rate limiting
//! - Circuit breakers
//! - Request tracing
//! - Audit logging

use crate::resilience::{CircuitBreaker, KeyedRateLimiter, RateLimitConfig};
use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

/// Fortune 5 API middleware stack
pub struct Fortune5Middleware {
    /// Enable Fortune 5 features
    enabled: bool,
}

impl Fortune5Middleware {
    /// Create new Fortune 5 middleware
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Build middleware stack
    pub fn build(
        self,
    ) -> ServiceBuilder<
        impl tower::Layer<
            tower::util::BoxService<
                axum::http::Request<axum::body::Body>,
                axum::http::Response<axum::body::Body>,
                axum::Error,
            >,
        >,
    > {
        ServiceBuilder::new()
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http())
    }
}

/// Authentication middleware
pub async fn auth_middleware(
    headers: HeaderMap,
    request: Request<Body>,
    next: Next<Body>,
) -> Result<Response, StatusCode> {
    // Extract authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    // Validate token format (Bearer token required)
    // Note: Full SPIFFE/SPIRE integration would validate mTLS certificates here
    if !auth_header.starts_with("Bearer ") {
        warn!("Invalid authorization header format");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Continue request
    Ok(next.run(request).await)
}

/// Global rate limiter for API middleware (per-client rate limiting)
static RATE_LIMITER: std::sync::OnceLock<Arc<KeyedRateLimiter<String>>> =
    std::sync::OnceLock::new();

fn get_rate_limiter() -> Arc<KeyedRateLimiter<String>> {
    RATE_LIMITER
        .get_or_init(|| {
            let config = RateLimitConfig {
                max_requests: 100,
                window_seconds: 60,
                burst_size: Some(20),
            };
            // KeyedRateLimiter::new should never fail with valid config
            // If it does, we panic as this is a critical initialization failure
            match KeyedRateLimiter::new("api_middleware".to_string(), config) {
                Ok(limiter) => Arc::new(limiter),
                Err(e) => {
                    tracing::error!("Failed to create rate limiter: {}", e);
                    panic!("Failed to create rate limiter: {} - this is a critical initialization failure", e);
                }
            }
        })
        .clone()
}

/// Rate limiting middleware
///
/// Implements per-client rate limiting using governor library.
/// Extracts client identifier from request headers (x-forwarded-for, x-real-ip, or remote address)
/// and applies rate limits per client.
pub async fn rate_limit_middleware(
    request: Request<Body>,
    next: Next<Body>,
) -> Result<Response, StatusCode> {
    // Extract client identifier from request
    let client_id = extract_client_id(&request);

    // Check rate limit for this client
    let limiter = get_rate_limiter();
    if limiter.check_key(&client_id).is_err() {
        warn!(
            client_id = %client_id,
            "Rate limit exceeded for client"
        );
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Rate limit passed, continue with request
    Ok(next.run(request).await)
}

/// Extract client identifier from request
///
/// Tries to extract client IP from headers in order:
/// 1. x-forwarded-for (first IP if multiple)
/// 2. x-real-ip
/// 3. Remote address from extensions
/// 4. Falls back to "unknown" if none available
fn extract_client_id(request: &Request<Body>) -> String {
    // Try x-forwarded-for header (first IP if comma-separated list)
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            let first_ip = forwarded_str.split(',').next().unwrap_or("").trim();
            if !first_ip.is_empty() {
                return first_ip.to_string();
            }
        }
    }

    // Try x-real-ip header
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            if !real_ip_str.is_empty() {
                return real_ip_str.to_string();
            }
        }
    }

    // Try remote address from extensions (set by reverse proxy)
    if let Some(remote_addr) = request.extensions().get::<std::net::SocketAddr>() {
        return remote_addr.ip().to_string();
    }

    // Fallback to "unknown" if no client identifier found
    "unknown".to_string()
}

/// Global circuit breaker for API middleware
static CIRCUIT_BREAKER: std::sync::OnceLock<Arc<CircuitBreaker>> = std::sync::OnceLock::new();

fn get_circuit_breaker() -> Arc<CircuitBreaker> {
    CIRCUIT_BREAKER
        .get_or_init(|| {
            Arc::new(CircuitBreaker::new(
                5,                       // failure_threshold: open after 5 failures
                Duration::from_secs(60), // timeout: 60 seconds before half-open
            ))
        })
        .clone()
}

/// Circuit breaker middleware
///
/// Implements circuit breaker pattern to prevent cascading failures.
/// Tracks request failures and opens circuit after threshold is exceeded.
/// Automatically transitions to half-open after timeout to test recovery.
pub async fn circuit_breaker_middleware(
    request: Request<Body>,
    next: Next<Body>,
) -> Result<Response, StatusCode> {
    let circuit_breaker = get_circuit_breaker();

    // Execute request with circuit breaker protection
    // The circuit breaker's execute() method handles state transitions
    // and tracks failures automatically
    let result = circuit_breaker
        .execute(|| async {
            let response = next.run(request).await;
            // Check if response indicates failure
            if response.status().is_server_error() {
                Err(crate::error::WorkflowError::ExternalSystem(format!(
                    "Server error: {}",
                    response.status()
                )))
            } else {
                Ok(response)
            }
        })
        .await;

    match result {
        Ok(response) => Ok(response),
        Err(e) => {
            warn!(
                error = %e,
                "Circuit breaker rejected request"
            );
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// Request tracing middleware
pub async fn tracing_middleware(request: Request<Body>, next: Next<Body>) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();

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
    request: Request<Body>,
    next: Next<Body>,
) -> Response {
    let user = headers
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let path = request.uri().path().to_string();
    let method = request.method().to_string();

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
