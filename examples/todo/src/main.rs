//! todo-example: A comprehensive example demonstrating MontRS features.
//! This application integrates signals, schema validation, and the ORM layer
//! to build a simple but functional Todo management system.

use async_trait::async_trait;
use montrs_core::{AppConfig, AppSpec, Module, ModuleContext, Router, Signal, Target, TypedEnv};
use montrs_orm::{DbBackend, FromRow, SqliteBackend};
use montrs_schema::Schema;
use serde::{Deserialize, Serialize};

// 1. Define the Schema for creating a Todo.
// The #[derive(Schema)] macro generates the validate() method.
#[derive(Debug, Serialize, Deserialize, Schema)]
pub struct CreateTodo {
    #[schema(min_len = 3)]
    pub title: String,
}

// Data model representing a Todo item in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: i32,
    pub title: String,
    pub completed: bool,
}

// Implement FromRow to allow the ORM to map database results to the Todo struct.
impl FromRow for Todo {
    fn from_row_sqlite(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            title: row.get(1)?,
            completed: row.get(2)?,
        })
    }

    fn from_row_postgres(_row: &tokio_postgres::Row) -> Result<Self, montrs_orm::DbError> {
        // Skeleton for now, Postgres integration is planned for v0.2
        Err(montrs_orm::DbError::Query(
            "Postgres not fully implemented in example".to_string(),
        ))
    }
}

// Custom error type for our application.
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Database error: {0}")]
    Db(#[from] montrs_orm::DbError),
    #[error("Generic error: {0}")]
    Generic(String),
}

// 2. Define the Application Configuration.
// This holds shared resources like the database connection.
pub struct MyConfig {
    pub db: SqliteBackend,
}

impl AppConfig for MyConfig {
    type Error = MyError;
    type Env = TypedEnv;
}

// 3. Define a Module for Todo logic.
// Modules are the primary unit of modularity in MontRS.
pub struct TodoModule;

#[async_trait]
impl Module<MyConfig> for TodoModule {
    fn name(&self) -> &'static str {
        "todo"
    }

    async fn init(
        &self,
        _ctx: &mut ModuleContext<MyConfig>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("TodoModule initialized");
        Ok(())
    }

    fn register_routes(&self, _router: &mut Router<MyConfig>) {
        // Here you would register loaders for fetching todos and actions for adding/removing them.
        println!("Routes registered for TodoModule");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the SQLite database in memory for this example.
    let db = SqliteBackend::new(":memory:")?;
    db.execute(
        "CREATE TABLE IF NOT EXISTS todos (id INTEGER PRIMARY KEY, title TEXT, completed BOOLEAN)",
        &[],
    )
    .await?;

    let config = MyConfig { db };
    let env = TypedEnv {};

    // 4. Bootstrap the Application Specification (AppSpec).
    // This is the blueprint of our application.
    let spec = AppSpec::new(config, env)
        .with_target(Target::Server)
        .with_module(Box::new(TodoModule));

    // Initialize modules (Simplified manual bootstrap for the example).
    for _module in &spec.modules {
        let _ctx = ModuleContext {
            config: &spec.config,
            env: &spec.env,
        };
        // _module.init(&mut ctx).await?;
    }

    // 5. Demonstrate Reactivity via Signals.
    // Changing a signal's value would normally trigger dependent effects.
    let counter = Signal::new(0);
    println!("Initial counter: {}", counter.get());

    counter.set(10);
    println!("Updated counter: {}", counter.get());

    // 6. Demonstrate Schema Validation.
    // The generated validate() method checks our min_len constraint.
    let valid_todo = CreateTodo {
        title: "Buy milk".to_string(),
    };
    let invalid_todo = CreateTodo {
        title: "a".to_string(),
    };

    println!("Valid todo check: {:?}", valid_todo.validate());
    println!("Invalid todo check: {:?}", invalid_todo.validate());

    // 7. Demonstrate ORM operations.
    // We can execute SQL and query typed results directly from our config's database backend.
    spec.config
        .db
        .execute(
            "INSERT INTO todos (title, completed) VALUES (?, ?)",
            &[&"Learn MontRS", &false],
        )
        .await?;
    let todos: Vec<Todo> = spec
        .config
        .db
        .query("SELECT id, title, completed FROM todos", &[])
        .await?;
    println!("Todos in DB: {:?}", todos);

    Ok(())
}
