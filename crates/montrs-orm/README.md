# montrs-orm

A trait-driven, async-first ORM layer for the MontRS framework.

## Overview

`montrs-orm` provides a unified interface for interacting with different database backends. It abstracts away the differences between database drivers while maintaining performance and type safety.

## Key Features

- **Multi-Backend support**:
    - **SQLite**: Using `rusqlite`.
    - **PostgreSQL**: Using `tokio-postgres` and `deadpool-postgres`.
- **Async API**: All database operations are `async` from the ground up.
- **Unified Parameters**: The `ToSql` trait allows passing parameters to different backends using a single API.
- **Type-Safe Mapping**: The `FromRow` trait ensures database rows are correctly mapped to Rust structs.

## Usage

```rust
use montrs_orm::{DbBackend, SqliteBackend};

let db = SqliteBackend::new(":memory:")?;
db.execute("CREATE TABLE users (id INTEGER, name TEXT)", &[]).await?;
```
