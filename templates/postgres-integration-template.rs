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
pub async fn execute_query(
    pool: &Pool,
    query: &str,
    params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
) -> Result<Vec<Row>, String> {
    // Get connection from pool
    let client = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    // Execute query
    client
        .query(query, params)
        .await
        .map_err(|e| format!("Query failed: {}", e))
}

/// Execute INSERT/UPDATE/DELETE and return affected rows
pub async fn execute_update(
    pool: &Pool,
    query: &str,
    params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
) -> Result<u64, String> {
    let client = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    client
        .execute(query, params)
        .await
        .map_err(|e| format!("Update failed: {}", e))
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
    // Get connection
    let mut client = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    // Begin transaction
    let transaction = client
        .transaction()
        .await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // Execute operations
    let result = transaction_fn(&transaction).await;

    match result {
        Ok(value) => {
            // Commit transaction
            transaction
                .commit()
                .await
                .map_err(|e| format!("Failed to commit: {}", e))?;
            Ok(value)
        }
        Err(e) => {
            // Rollback transaction
            transaction
                .rollback()
                .await
                .map_err(|err| format!("Failed to rollback: {}", err))?;
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

// TODO: Add telemetry
// use knhk_otel::{Tracer, SpanStatus};
//
// pub async fn execute_query_with_telemetry(...) -> Result<Vec<Row>, String> {
//     let mut tracer = Tracer::new();
//     let span = tracer.start_span("db.query".to_string(), None);
//     tracer.add_attribute(span.clone(), "db.query".to_string(), query.to_string());
//
//     let result = execute_query(pool, query, params).await;
//
//     match &result {
//         Ok(rows) => {
//             tracer.add_attribute(span.clone(), "db.rows".to_string(), rows.len().to_string());
//             tracer.end_span(span, SpanStatus::Ok)
//         }
//         Err(e) => {
//             tracer.add_attribute(span.clone(), "error".to_string(), e.to_string());
//             tracer.end_span(span, SpanStatus::Error)
//         }
//     }
//
//     result
// }

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
