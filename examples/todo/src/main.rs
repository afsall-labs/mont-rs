use montrs_core::{AppConfig, AppSpec, Module, ModuleContext, Router, Target, TypedEnv, Signal};
use montrs_schema::Schema;
use montrs_orm::{DbBackend, SqliteBackend, FromRow};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

// 1. Define the Schema
#[derive(Debug, Serialize, Deserialize, Schema)]
pub struct CreateTodo {
    #[schema(min_len = 3)]
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: i32,
    pub title: String,
    pub completed: bool,
}

impl FromRow for Todo {
    fn from_row_sqlite(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            title: row.get(1)?,
            completed: row.get(2)?,
        })
    }

    fn from_row_postgres(_row: &tokio_postgres::Row) -> Result<Self, montrs_orm::DbError> {
        // Skeleton for now
        Err(montrs_orm::DbError::Query("Postgres not fully implemented in example".to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Database error: {0}")]
    Db(#[from] montrs_orm::DbError),
    #[error("Generic error: {0}")]
    Generic(String),
}

// 2. Define the Application Config
pub struct MyConfig {
    pub db: SqliteBackend,
}

impl AppConfig for MyConfig {
    type Error = MyError;
    type Env = TypedEnv;
}

// 3. Define a Module
pub struct TodoModule;

#[async_trait]
impl Module<MyConfig> for TodoModule {
    fn name(&self) -> &'static str {
        "todo"
    }

    async fn init(&self, _ctx: &mut ModuleContext<MyConfig>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("TodoModule initialized");
        Ok(())
    }

    fn register_routes(&self, _router: &mut Router<MyConfig>) {
        // Register loaders and actions here (mocked for this example)
        println!("Routes registered for TodoModule");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize DB
    let db = SqliteBackend::new(":memory:")?;
    db.execute(
        "CREATE TABLE IF NOT EXISTS todos (id INTEGER PRIMARY KEY, title TEXT, completed BOOLEAN)",
        &[],
    ).await?;

    let config = MyConfig { db };
    let env = TypedEnv {};
    
    // 4. Bootstrap AppSpec
    let spec = AppSpec::new(config, env)
        .with_target(Target::Server)
        .with_module(Box::new(TodoModule));

    // Initialize modules
    for _module in &spec.modules {
        let _ctx = ModuleContext {
            config: &spec.config,
            env: &spec.env,
        };
        // In a real runtime, we'd call init here
    }

    // 5. Demonstrate Reactivity
    let counter = Signal::new(0);
    println!("Initial counter: {}", counter.get());

    counter.set(10);
    println!("Updated counter: {}", counter.get());

    // 6. Demonstrate Schema Validation
    let valid_todo = CreateTodo { title: "Buy milk".to_string() };
    let invalid_todo = CreateTodo { title: "a".to_string() };

    println!("Valid todo check: {:?}", valid_todo.validate());
    println!("Invalid todo check: {:?}", invalid_todo.validate());

    // 7. Demonstrate ORM
    spec.config.db.execute("INSERT INTO todos (title, completed) VALUES (?, ?)", &[&"Learn MontRS", &false]).await?;
    let todos: Vec<Todo> = spec.config.db.query("SELECT id, title, completed FROM todos", &[]).await?;
    println!("Todos in DB: {:?}", todos);

    Ok(())
}
