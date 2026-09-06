use leptos::prelude::*;
use montrs_core::{
    AppConfig, AppSpec, EnvConfig, EnvError, Plate, PlateContext, Route, RouteAction,
    RouteContext, RouteError, RouteLoader, RouteParams, RouteView, Router, RouterOutlet, Target,
};
use montrs_orm::{DbBackend, FromRow, SqliteBackend};
use montrs_validator::Validator;
use montrs_ui::prelude::*;
use montrs_icons::*;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use montrs_state::Store;
use montrs_table_core::{Row, Table};

pub fn build_spec() -> AppSpec<MyConfig> {
    let mut spec = AppSpec::new(
        MyConfig {
            db_url: ":memory:".to_string(),
        },
        MyEnv,
    )
    .with_target(Target::Web)
    .with_plate(TodoPlate);
    TodoPlate.register_routes(&mut spec.router);
    spec
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    let spec = build_spec();
    leptos::mount::hydrate_body(move || {
        provide_context(spec.router);
        App()
    });
}

#[component]
pub fn Shell() -> impl IntoView {
    let leptos_options = use_context::<LeptosOptions>()
        .expect("LeptosOptions must be provided by the SSR server");

    view! {
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
                <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32.png" />
                <link rel="apple-touch-icon" href="/favicon-180.png" />
                <link rel="stylesheet" href="/main.css" />
                <title>"MontRS Todo"</title>
                <script>
                    "(function(){try{var t=localStorage.getItem('montrs-theme');var d=t?t==='dark':window.matchMedia('(prefers-color-scheme: dark)').matches;if(d)document.documentElement.classList.add('dark');}catch(e){}})();"
                </script>
                <HydrationScripts options=leptos_options />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let todos = Store::new(Vec::<String>::new(), |state: &Vec<String>, event: &String| {
        let mut next = state.clone();
        next.push(event.clone());
        Ok(next)
    });
    let table = Table::new(vec![Row { id: "example".into(), value: "Todo state" }]);
    provide_context(todos);
    provide_context(table);

    view! {
        <leptos_router::components::Router>
            <ThemeProvider>
            <div class="min-h-screen bg-background text-foreground">
                <header class="border-b border-border">
                    <div class="mx-auto flex h-16 max-w-2xl items-center gap-2 px-6">
                        <CheckCheckIcon class="h-6 w-6 text-primary" />
                        <span class="text-lg font-bold">"MontRS Todo"</span>
                    </div>
                </header>
                <main class="mx-auto max-w-2xl px-6 py-12">
                    {RouterOutlet::<MyConfig>()}
                </main>
            </div>
        </ThemeProvider>
        </leptos_router::components::Router>
    }
}

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum MyError {
    #[error("Database error: {0}")]
    Db(String),
    #[error("Generic error: {0}")]
    Generic(String),
}

#[derive(Clone)]
pub struct MyEnv;
impl EnvConfig for MyEnv {
    fn get_var(&self, key: &str) -> Result<String, EnvError> {
        match key {
            "DATABASE_URL" => Ok("sqlite::memory:".to_string()),
            _ => Err(EnvError::MissingKey(key.to_string())),
        }
    }
}

#[derive(Clone)]
pub struct MyConfig {
    pub db_url: String,
}
impl AppConfig for MyConfig {
    type Error = MyError;
    type Env = MyEnv;
}

#[derive(Debug, Clone, Serialize, Deserialize, Validator)]
pub struct CreateTodo {
    #[validator(min_len = 3)]
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: i32,
    pub title: String,
    pub completed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TodoParams {}
impl RouteParams for TodoParams {}

pub struct TodoLoader;
#[async_trait]
impl RouteLoader<TodoParams, MyConfig> for TodoLoader {
    type Output = Vec<Todo>;
    async fn load(
        &self,
        _ctx: RouteContext<'_, MyConfig>,
        _params: TodoParams,
    ) -> Result<Self::Output, RouteError> {
        Ok(vec![])
    }
    fn description(&self) -> &'static str {
        "Loads all todos"
    }
}

pub struct TodoAction;
#[async_trait]
impl RouteAction<TodoParams, MyConfig> for TodoAction {
    type Input = CreateTodo;
    type Output = Todo;
    async fn act(
        &self,
        _ctx: RouteContext<'_, MyConfig>,
        _params: TodoParams,
        _input: Self::Input,
    ) -> Result<Self::Output, RouteError> {
        Ok(Todo {
            id: 1,
            title: "New Todo".to_string(),
            completed: false,
        })
    }
    fn description(&self) -> &'static str {
        "Creates a new todo"
    }
}

pub struct TodoView;

impl RouteView for TodoView {
    fn render(&self) -> impl IntoView {
        let (count, set_count) = signal(0);
        view! {
            <div class="rounded-lg border border-border bg-card p-8">
                <div class="flex items-center gap-3">
                    <ListChecksIcon class="h-8 w-8 text-primary" />
                    <div>
                        <h1 class="text-2xl font-bold">"Todo Manager"</h1>
                        <p class="text-sm text-muted-foreground">
                            "Scaffolded Explicit Architecture example."
                        </p>
                    </div>
                </div>
                <div class="mt-8 flex items-center gap-4">
                    <button
                        on:click=move |_| set_count.update(|n| *n += 1)
                        class="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90"
                    >
                        <PlusIcon class="h-4 w-4" />
                        "Count: " {count}
                    </button>
                    <span class="text-sm text-muted-foreground">
                        "Click to increment"
                    </span>
                </div>
                <div class="mt-6 rounded-md bg-muted p-4">
                    <p class="text-xs text-muted-foreground">
                        "This example demonstrates: AppSpec, Plate, Route, Loader, Action, Validator, ORM, and montrs-ui components."
                    </p>
                </div>
            </div>
        }
    }
}

pub struct TodoRoute;

impl Route<MyConfig> for TodoRoute {
    type Params = TodoParams;
    type Loader = TodoLoader;
    type Action = TodoAction;
    type View = TodoView;

    fn path() -> &'static str {
        "/"
    }
    fn loader(&self) -> Self::Loader {
        TodoLoader
    }
    fn action(&self) -> Self::Action {
        TodoAction
    }
    fn view(&self) -> Self::View {
        TodoView
    }
}

pub struct TodoPlate;

#[async_trait]
impl Plate<MyConfig> for TodoPlate {
    fn name(&self) -> &'static str {
        "todo"
    }
    fn description(&self) -> &'static str {
        "Todo example with loaders and actions"
    }
    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }
    async fn init(
        &self,
        _ctx: &mut PlateContext<MyConfig>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
    fn register_routes(&self, router: &mut Router<MyConfig>) {
        router.register(TodoRoute);
    }
}
