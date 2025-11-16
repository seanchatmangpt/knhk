// Axum HTTP Server Template
// Ready-to-use HTTP server with middleware and telemetry
//
// Features:
// - Axum web framework
// - Middleware (logging, telemetry, errors)
// - Health check endpoints
// - Graceful shutdown
// - Error handling

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;

#[cfg(feature = "otel")]
use tracing::{debug, error, info, instrument, span, Level};

// ============================================================================
// Application State
// ============================================================================

#[derive(Clone)]
struct AppState {
    service_name: String,
    version: String,
    // Add your application state here (database pool, cache, etc.)
}

impl AppState {
    fn new(service_name: &str, version: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            version: version.to_string(),
        }
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct QueryRequest {
    sparql: String,
}

#[derive(Debug, Serialize)]
struct QueryResponse {
    result: bool,
    execution_time_ms: u128,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

// ============================================================================
// Error Handling
// ============================================================================

enum AppError {
    InvalidInput(String),
    ExecutionError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::ExecutionError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(ErrorResponse { error: error_message })).into_response()
    }
}

// ============================================================================
// Route Handlers
// ============================================================================

/// Health check endpoint
#[cfg_attr(feature = "otel", instrument(
    name = "knhk.http.health_check",
    skip(state),
    fields(
        knhk.operation.name = "health_check",
        knhk.operation.type = "http",
        service.name = %state.service_name
    )
))]
async fn health_check(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    #[cfg(feature = "otel")]
    debug!(service = %state.service_name, version = %state.version, "health_check_requested");

    Json(serde_json::json!({
        "status": "healthy",
        "service": state.service_name,
        "version": state.version,
    }))
}

/// Readiness check endpoint
#[cfg_attr(feature = "otel", instrument(
    name = "knhk.http.readiness_check",
    fields(
        knhk.operation.name = "readiness_check",
        knhk.operation.type = "http"
    )
))]
async fn readiness_check() -> Json<serde_json::Value> {
    #[cfg(feature = "otel")]
    debug!("readiness_check_requested");

    // Check dependencies (database, cache, etc.)
    // For now, always ready
    Json(serde_json::json!({
        "status": "ready",
    }))
}

/// Execute query endpoint
#[cfg_attr(feature = "otel", instrument(
    name = "knhk.http.execute_query",
    skip(_state, request),
    fields(
        knhk.operation.name = "execute_query",
        knhk.operation.type = "http",
        query.length = request.sparql.len()
    )
))]
async fn execute_query(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, AppError> {
    // Validate input
    if request.sparql.is_empty() {
        #[cfg(feature = "otel")]
        error!("empty_query_rejected");
        return Err(AppError::InvalidInput("Query cannot be empty".to_string()));
    }

    #[cfg(feature = "otel")]
    debug!(query_length = request.sparql.len(), "executing_query");

    // Execute query (simulated)
    let start = std::time::Instant::now();

    // Simulate query execution
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let result = true;
    let execution_time = start.elapsed().as_millis();

    #[cfg(feature = "otel")]
    info!(
        result = %result,
        execution_time_ms = %execution_time,
        "query_executed_successfully"
    );

    Ok(Json(QueryResponse {
        result,
        execution_time_ms: execution_time,
    }))
}

// ============================================================================
// Middleware
// ============================================================================

/// Logging middleware
async fn logging_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    #[cfg(feature = "otel")]
    {
        let _span = span!(
            Level::INFO,
            "knhk.http.request",
            http.method = %method,
            http.uri = %uri,
            knhk.operation.type = "http"
        );
        let _enter = _span.enter();

        debug!(method = %method, uri = %uri, "incoming_request");

        let response = next.run(request).await;
        let status = response.status();

        info!(
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            "request_completed"
        );

        response
    }

    #[cfg(not(feature = "otel"))]
    {
        println!("→ {} {}", method, uri);

        let response = next.run(request).await;

        println!("← {} {} {}", method, uri, response.status());

        response
    }
}

// ============================================================================
// Application Setup
// ============================================================================

/// Create application router
fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check endpoints
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        // API endpoints
        .route("/api/v1/query", post(execute_query))
        // Middleware
        .layer(axum::middleware::from_fn(logging_middleware))
        // State
        .with_state(state)
}

/// Graceful shutdown signal
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("\nReceived Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            println!("\nReceived SIGTERM, shutting down gracefully...");
        },
    }
}

// ============================================================================
// Main: Start HTTP Server
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Axum HTTP Server Template ===\n");

    // Create application state
    let state = Arc::new(AppState::new("knhk-http-server", "1.0.0"));

    // Create application
    let app = create_app(state);

    // Bind to address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server starting on http://{}", addr);
    println!();
    println!("Available endpoints:");
    println!("  GET  http://{}/health", addr);
    println!("  GET  http://{}/ready", addr);
    println!("  POST http://{}/api/v1/query", addr);
    println!();
    println!("Press Ctrl+C to stop\n");

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    println!("✅ Server stopped gracefully");

    Ok(())
}

// ============================================================================
// Example Client Requests
// ============================================================================

// curl -X GET http://localhost:3000/health
// Response: {"status":"healthy","service":"knhk-http-server","version":"1.0.0"}

// curl -X GET http://localhost:3000/ready
// Response: {"status":"ready"}

// curl -X POST http://localhost:3000/api/v1/query \
//   -H "Content-Type: application/json" \
//   -d '{"sparql":"ASK { ?s ?p ?o }"}'
// Response: {"result":true,"execution_time_ms":10}

// curl -X POST http://localhost:3000/api/v1/query \
//   -H "Content-Type: application/json" \
//   -d '{"sparql":""}'
// Response: {"error":"Query cannot be empty"}

// ============================================================================
// Production Enhancements
// ============================================================================

// ✅ Telemetry: IMPLEMENTED
//
// Telemetry has been integrated using the `tracing` crate with OpenTelemetry support.
// Each handler and middleware now includes:
// - Instrumentation using #[instrument] attribute for async handlers
// - Span creation for HTTP requests with method, URI, and status tracking
// - Structured logging with debug/info/error macros
// - Essential attributes only (minimal overhead)
//
// To use telemetry in production:
// 1. Build with the "otel" feature: `cargo build --features otel`
// 2. Initialize tracing subscriber with OTLP exporter in main()
// 3. Optionally add tower_http::trace::TraceLayer for additional middleware tracing
//
// Example usage:
// use tower_http::trace::TraceLayer;
//
// Router::new()
//     .layer(TraceLayer::new_for_http())  // Optional: additional HTTP tracing
//
// The telemetry follows KNHK's instrumentation principles:
// - Schema-first approach (define spans in OTel schema)
// - Service boundary instrumentation
// - Context propagation through parent-child spans
// - Essential attributes only
// - Performance budget compliance (minimal overhead)

// TODO: Add CORS middleware
// use tower_http::cors::{CorsLayer, Any};
//
// let cors = CorsLayer::new()
//     .allow_origin(Any)
//     .allow_methods([Method::GET, Method::POST])
//     .allow_headers(Any);
//
// Router::new()
//     .layer(cors)

// TODO: Add rate limiting
// use tower::limit::RateLimitLayer;
// use std::time::Duration;
//
// let rate_limit = RateLimitLayer::new(100, Duration::from_secs(1));
//
// Router::new()
//     .layer(rate_limit)

// TODO: Add request timeout
// use tower_http::timeout::TimeoutLayer;
//
// let timeout = TimeoutLayer::new(Duration::from_secs(30));
//
// Router::new()
//     .layer(timeout)

// TODO: Add compression
// use tower_http::compression::CompressionLayer;
//
// Router::new()
//     .layer(CompressionLayer::new())

// TODO: Add authentication
// async fn auth_middleware(...) -> Result<(), StatusCode> {
//     // Verify JWT token or API key
// }

// Dependencies (add to Cargo.toml):
// [dependencies]
// axum = "0.7"
// tokio = { version = "1", features = ["full"] }
// serde = { version = "1", features = ["derive"] }
// serde_json = "1"
// tower-http = { version = "0.5", features = ["trace", "cors", "timeout", "compression"] }
// tower = "0.4"
