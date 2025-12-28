//! montrs-orm: A trait-driven ORM layer for MontRS.
//! This crate defines the DbBackend trait and provides implementations for
//! SQLite and PostgreSQL, enabling unified database access.

use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use async_trait::async_trait;
use deadpool_postgres::{Pool, Config, Runtime};
use tokio_postgres::NoTls;

/// Errors that can occur during database operations.
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    Connection(String),
    #[error("Query execution error: {0}")]
    Query(String),
    #[error("Migration error: {0}")]
    Migration(String),
}

/// A unified trait for parameters to support multiple backends.
/// Bridges the gap between rusqlite::ToSql and tokio_postgres::types::ToSql.
pub trait ToSql: Send + Sync {
    /// Returns a reference that can be used by rusqlite.
    fn as_rusqlite(&self) -> &dyn rusqlite::ToSql;
}

// Implementations for common types to be used as query parameters.
impl ToSql for String {
    fn as_rusqlite(&self) -> &dyn rusqlite::ToSql { self }
}
impl ToSql for i32 {
    fn as_rusqlite(&self) -> &dyn rusqlite::ToSql { self }
}
impl ToSql for bool {
    fn as_rusqlite(&self) -> &dyn rusqlite::ToSql { self }
}
impl ToSql for &str {
    fn as_rusqlite(&self) -> &dyn rusqlite::ToSql { self }
}

/// Trait for mapping database rows to Rust types.
/// Requires backend-specific mapping methods.
pub trait FromRow: Sized {
    /// Maps a rusqlite row to the implementor type.
    fn from_row_sqlite(row: &rusqlite::Row) -> rusqlite::Result<Self>;
    /// Maps a tokio-postgres row to the implementor type.
    fn from_row_postgres(row: &tokio_postgres::Row) -> Result<Self, DbError>;
}

/// The core abstraction for database backends.
/// Provides async methods for executing and querying.
#[async_trait]
pub trait DbBackend: Send + Sync + 'static {
    /// Executes a non-query SQL statement (INSERT, UPDATE, DELETE).
    async fn execute(&self, sql: &str, params: &[&dyn ToSql]) -> Result<usize, DbError>;
    /// Executes a query SQL statement and returns a vector of results.
    async fn query<T: FromRow>(&self, sql: &str, params: &[&dyn ToSql]) -> Result<Vec<T>, DbError>;
}

/// SQLite-specific database backend implementation.
/// Uses synchronous rusqlite under the hood with internal locking.
pub struct SqliteBackend {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteBackend {
    /// Creates a new SqliteBackend connecting to the specified path (or :memory:).
    pub fn new(path: &str) -> Result<Self, DbError> {
        let conn = if path == ":memory:" {
            Connection::open_in_memory()
        } else {
            Connection::open(path)
        }.map_err(|e| DbError::Connection(e.to_string()))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait]
impl DbBackend for SqliteBackend {
    async fn execute(&self, sql: &str, params: &[&dyn ToSql]) -> Result<usize, DbError> {
        let conn = self.conn.lock().unwrap();
        // Convert unified params to rusqlite-compatible params.
        let sqlite_params: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_rusqlite()).collect();
        conn.execute(sql, rusqlite::params_from_iter(sqlite_params))
            .map_err(|e| DbError::Query(e.to_string()))
    }

    async fn query<T: FromRow>(&self, sql: &str, params: &[&dyn ToSql]) -> Result<Vec<T>, DbError> {
        let conn = self.conn.lock().unwrap();
        let sqlite_params: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_rusqlite()).collect();
        let mut stmt = conn.prepare(sql).map_err(|e| DbError::Query(e.to_string()))?;
        let rows = stmt.query_map(rusqlite::params_from_iter(sqlite_params), |row| T::from_row_sqlite(row))
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| DbError::Query(e.to_string()))?);
        }
        Ok(results)
    }
}

/// PostgreSQL-specific database backend implementation.
/// Uses deadpool-postgres for async connection pooling.
pub struct PostgresBackend {
    pool: Pool,
}

impl PostgresBackend {
    /// Creates a new PostgresBackend with the provided configuration.
    pub fn new(config: Config) -> Result<Self, DbError> {
        let pool = config.create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| DbError::Connection(e.to_string()))?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl DbBackend for PostgresBackend {
    async fn execute(&self, sql: &str, _params: &[&dyn ToSql]) -> Result<usize, DbError> {
        let client = self.pool.get().await.map_err(|e| DbError::Connection(e.to_string()))?;
        // tokio-postgres requires different params handling, implementing skeleton
        client.execute(sql, &[])
            .await
            .map(|n| n as usize)
            .map_err(|e| DbError::Query(e.to_string()))
    }

    async fn query<T: FromRow>(&self, sql: &str, _params: &[&dyn ToSql]) -> Result<Vec<T>, DbError> {
        let client = self.pool.get().await.map_err(|e| DbError::Connection(e.to_string()))?;
        let rows = client.query(sql, &[])
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(T::from_row_postgres(&row)?);
        }
        Ok(results)
    }
}
