//! PostgreSQL connection utilities.
//!
//! This module provides functions for establishing and managing
//! connection pools to a PostgreSQL database using the `sqlx` crate.

use sqlx::postgres::PgPoolOptions;

/// Creates a connection pool for a PostgreSQL database.
///
/// # Arguments
///
/// * `url` - A connection string for the PostgreSQL server.
///
/// # Returns
///
/// A `Result` containing the `PgPool` on success or a `sqlx::Error` on failure.
/// The pool is configured with a maximum of 10 connections.
///
/// # Example
///
/// ```ignore
/// let pool = create_pool("postgresql://user:pass@localhost:5432/db").await?;
/// ```
pub async fn create_pool(url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await
}
