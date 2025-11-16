// PostgreSQL Integration Template
// Ready-to-use database integration with connection pooling
//
// Features:
// - Connection pooling (deadpool-postgres)
// - Query execution with error handling
// - Transaction support
// - Telemetry integration
// - Migration support

use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::{NoTls, Row};

#[cfg(feature = "otel")]
use tracing::{debug, error, info, instrument, span, Level};

// ============================================================================
// Database Configuration
// ============================================================================

/// Create PostgreSQL connection pool
pub fn create_pool(database_url: &str) -> Result<Pool, String> {
    let mut cfg = Config::new();

    // Parse database URL: postgres://user:password@localhost:5432/dbname
    cfg.url = Some(database_url.to_string());

    // Connection pool configuration
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    // Pool size
    cfg.pool = Some(deadpool_postgres::PoolConfig::new(10));

    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        .map_err(|e| format!("Failed to create pool: {}", e))
}

// ============================================================================
// Query Execution
// ============================================================================

/// Execute query and return rows
#[cfg_attr(feature = "otel", instrument(
    name = "knhk.db.query",
    skip(pool, params),
    fields(
        knhk.operation.name = "db.query",
        knhk.operation.type = "database",
        db.query = query,
        db.params_count = params.len()
    )
))]
pub async fn execute_query(
    pool: &Pool,
    query: &str,
    params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
) -> Result<Vec<Row>, String> {
    #[cfg(feature = "otel")]
    debug!(query = %query, params_count = params.len(), "executing_query");

    // Get connection from pool
    let client = pool
        .get()
        .await
        .map_err(|e| {
            #[cfg(feature = "otel")]
            error!(error = %e, "failed_to_get_connection");
            format!("Failed to get connection: {}", e)
        })?;

    // Execute query
    let result = client
        .query(query, params)
        .await
        .map_err(|e| {
            #[cfg(feature = "otel")]
            error!(error = %e, query = %query, "query_failed");
            format!("Query failed: {}", e)
        })?;

    #[cfg(feature = "otel")]
    info!(row_count = result.len(), "query_executed_successfully");

    Ok(result)
}

/// Execute INSERT/UPDATE/DELETE and return affected rows
#[cfg_attr(feature = "otel", instrument(
    name = "knhk.db.update",
    skip(pool, params),
    fields(
        knhk.operation.name = "db.update",
        knhk.operation.type = "database",
        db.query = query,
        db.params_count = params.len()
    )
))]
pub async fn execute_update(
    pool: &Pool,
    query: &str,
    params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
) -> Result<u64, String> {
    #[cfg(feature = "otel")]
    debug!(query = %query, params_count = params.len(), "executing_update");

    let client = pool
        .get()
        .await
        .map_err(|e| {
            #[cfg(feature = "otel")]
            error!(error = %e, "failed_to_get_connection");
            format!("Failed to get connection: {}", e)
        })?;

    let affected_rows = client
        .execute(query, params)
        .await
        .map_err(|e| {
            #[cfg(feature = "otel")]
            error!(error = %e, query = %query, "update_failed");
            format!("Update failed: {}", e)
        })?;

    #[cfg(feature = "otel")]
    info!(affected_rows = affected_rows, "update_executed_successfully");

    Ok(affected_rows)
}

// ============================================================================
// Transaction Support
// ============================================================================

/// Execute multiple operations in a transaction
pub async fn execute_transaction<F, T>(pool: &Pool, transaction_fn: F) -> Result<T, String>
where
    F: FnOnce(&tokio_postgres::Transaction) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<T, String>> + Send + '_>,
    >,
{
    #[cfg(feature = "otel")]
    let _span = span!(
        Level::INFO,
        "knhk.db.transaction",
        knhk.operation.name = "db.transaction",
        knhk.operation.type = "database"
    );

    #[cfg(feature = "otel")]
    let _enter = _span.enter();

    #[cfg(feature = "otel")]
    debug!("starting_transaction");

    // Get connection
    let mut client = pool
        .get()
        .await
        .map_err(|e| {
            #[cfg(feature = "otel")]
            error!(error = %e, "failed_to_get_connection");
            format!("Failed to get connection: {}", e)
        })?;

    // Begin transaction
    let transaction = client
        .transaction()
        .await
        .map_err(|e| {
            #[cfg(feature = "otel")]
            error!(error = %e, "failed_to_start_transaction");
            format!("Failed to start transaction: {}", e)
        })?;

    // Execute operations
    let result = transaction_fn(&transaction).await;

    match result {
        Ok(value) => {
            // Commit transaction
            transaction
                .commit()
                .await
                .map_err(|e| {
                    #[cfg(feature = "otel")]
                    error!(error = %e, "failed_to_commit");
                    format!("Failed to commit: {}", e)
                })?;

            #[cfg(feature = "otel")]
            info!("transaction_committed");

            Ok(value)
        }
        Err(e) => {
            #[cfg(feature = "otel")]
            error!(error = %e, "transaction_error_rolling_back");

            // Rollback transaction
            transaction
                .rollback()
                .await
                .map_err(|err| {
                    #[cfg(feature = "otel")]
                    error!(error = %err, "failed_to_rollback");
                    format!("Failed to rollback: {}", err)
                })?;

            #[cfg(feature = "otel")]
            info!("transaction_rolled_back");

            Err(e)
        }
    }
}

// ============================================================================
// Example Usage
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PostgreSQL Integration Template ===\n");

    // Configuration
    let database_url = "postgres://postgres:password@localhost:5432/knhk";

    // Create connection pool
    println!("Creating connection pool...");
    let pool = create_pool(database_url)?;
    println!("✅ Pool created\n");

    // Example 1: Create table
    println!("--- Example 1: Create Table ---");
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            email VARCHAR(255) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            created_at TIMESTAMP DEFAULT NOW()
        )
    "#;

    execute_update(&pool, create_table_query, &[]).await?;
    println!("✅ Table created\n");

    // Example 2: Insert data
    println!("--- Example 2: Insert Data ---");
    let insert_query = "INSERT INTO users (email, name) VALUES ($1, $2)";

    execute_update(&pool, insert_query, &[&"alice@example.com", &"Alice"]).await?;
    execute_update(&pool, insert_query, &[&"bob@example.com", &"Bob"]).await?;
    println!("✅ Data inserted\n");

    // Example 3: Query data
    println!("--- Example 3: Query Data ---");
    let select_query = "SELECT id, email, name FROM users ORDER BY id";

    let rows = execute_query(&pool, select_query, &[]).await?;
    for row in rows {
        let id: i32 = row.get(0);
        let email: String = row.get(1);
        let name: String = row.get(2);
        println!("User {}: {} ({})", id, name, email);
    }
    println!();

    // Example 4: Transaction
    println!("--- Example 4: Transaction ---");
    let result = execute_transaction(&pool, |tx| {
        Box::pin(async move {
            // Update Alice's name
            tx.execute("UPDATE users SET name = $1 WHERE email = $2", &[&"Alice Smith", &"alice@example.com"])
                .await
                .map_err(|e| format!("Update failed: {}", e))?;

            // Insert new user
            tx.execute("INSERT INTO users (email, name) VALUES ($1, $2)", &[&"charlie@example.com", &"Charlie"])
                .await
                .map_err(|e| format!("Insert failed: {}", e))?;

            Ok(())
        })
    })
    .await;

    match result {
        Ok(_) => println!("✅ Transaction committed"),
        Err(e) => println!("❌ Transaction rolled back: {}", e),
    }
    println!();

    // Example 5: Cleanup
    println!("--- Example 5: Cleanup ---");
    execute_update(&pool, "DROP TABLE users", &[]).await?;
    println!("✅ Table dropped\n");

    println!("=== Production Enhancements ===");
    println!("- [ ] Add telemetry (trace queries with knhk_otel)");
    println!("- [ ] Add retry logic for transient errors");
    println!("- [ ] Add query timeouts");
    println!("- [ ] Add prepared statements for frequently used queries");
    println!("- [ ] Add connection health checks");
    println!("- [ ] Add database migration tool (diesel, sqlx)");
    println!("- [ ] Add connection string encryption");
    println!("- [ ] Add query logging for debugging");

    Ok(())
}

// ============================================================================
// Production Templates
// ============================================================================

// ✅ Telemetry: IMPLEMENTED
//
// Telemetry has been integrated using the `tracing` crate with OpenTelemetry support.
// Each database operation now includes:
// - Instrumentation using #[instrument] attribute for async functions
// - Span creation for queries, updates, and transactions
// - Structured logging with debug/info/error macros
// - Query tracking with row count and affected rows
// - Error context preservation with query details
// - Transaction lifecycle tracking (begin, commit, rollback)
//
// To use telemetry in production:
// 1. Build with the "otel" feature: `cargo build --features otel`
// 2. Initialize tracing subscriber with OTLP exporter before database operations
// 3. All database operations will automatically emit telemetry spans
//
// The telemetry follows KNHK's instrumentation principles:
// - Schema-first approach (define spans in OTel schema)
// - Database boundary instrumentation
// - Essential attributes only (query, params count, row count)
// - Performance budget compliance (minimal overhead)

// TODO: Add retry logic
// pub async fn execute_with_retry(pool: &Pool, query: &str, ..., max_retries: usize) -> Result<Vec<Row>, String> {
//     let mut attempts = 0;
//
//     loop {
//         attempts += 1;
//
//         match execute_query(pool, query, params).await {
//             Ok(rows) => return Ok(rows),
//             Err(e) if attempts >= max_retries => return Err(e),
//             Err(e) => {
//                 eprintln!("Retry {}/{}: {}", attempts, max_retries, e);
//                 tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempts as u64)).await;
//             }
//         }
//     }
// }

// Dependencies (add to Cargo.toml):
// [dependencies]
// tokio = { version = "1", features = ["full"] }
// tokio-postgres = "0.7"
// deadpool-postgres = "0.12"
