//! PostgreSQL connection pool
//!
//! ZFSS local PostgreSQL connection pool.
//! The physical database may retain a legacy `dataforge` name, which conveys no DataForge authority.

use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Create a new PostgreSQL connection pool
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .connect(database_url)
        .await
        .context("Failed to connect to PostgreSQL")?;

    // Test the connection
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .context("Failed to execute test query")?;

    Ok(pool)
}

/// Run migrations (embedded SQL)
pub async fn run_migrations(pool: &PgPool, migration_sql: &str) -> Result<()> {
    // Split by semicolon and execute each statement
    for statement in migration_sql.split(';') {
        let stmt = statement.trim();
        if stmt.is_empty() || stmt.starts_with("--") {
            continue;
        }

        sqlx::query(stmt).execute(pool).await.context(format!(
            "Failed to execute migration: {}",
            &stmt[..stmt.len().min(50)]
        ))?;
    }

    Ok(())
}

/// Check if the database is healthy
pub async fn health_check(pool: &PgPool) -> Result<bool> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map(|_| true)
        .context("Database health check failed")
}
