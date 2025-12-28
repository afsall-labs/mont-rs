# **MontRS Framework Product Requirements Document (PRD)**

**Version:** 0.1 (draft)  
**License:** MIT  
**Scope:** Full framework spec \+ `create-mont-app` CLI \+ repo and template structure \+ docs \+ CI/CD & Cloudflare deployment guidance

# **1\. Executive summary**

MontRS is a Rust-native, trait-driven, deterministic web framework that blends the engineering strengths of Leptos (fine-grained reactivity), Remix (web-native routing/HTTP mental model), Dioxus (ergonomic multi-target components), Yew (architecture discipline), Substrate/Setheum (trait-first modularity, deterministic initialization, test harness), and Drizzle (minimal-abstraction SQL ergonomics). MontRS is built for teams that value compile-time correctness, explicit boundaries, deterministic initialization across targets, test-first design, and minimal runtime overhead.

This PRD defines the technical architecture, core runtime API surfaces, module system, schema model, ORM expectations, testing & mocking, the `create-mont-app` CLI scaffolding tool (including default template), repository and app folder conventions, documentation layout, CI/CD & Cloudflare deployment guidance, and versioning and release practices to produce a production-ready MontRS v0.1.0.

# **2\. Guiding philosophy & design constraints**

**Principles (must hold for v0.1):**

* **Compile-time correctness:** Type-driven design; traits and typed configs everywhere. Prefer compiler guarantees over runtime heuristics.  
* **Deterministic execution:** AppSpec is the single source of initialization; tests/mocks instantiate AppSpec to guarantee reproducible runs.  
* **Explicit boundaries:** Reads (loaders) vs writes (actions); mutation is explicit; reactivity is automatic but observable.  
* **Trait-driven modularity:** Modules (pallets → modules) declare their contract via traits; AppSpec composes them; modules must be replaceable/mockable.  
* **Minimal abstraction & SQL-first:** ORM mirrors Drizzle — keep SQL explicit and available; helpers for newbies optional.  
* **Test-first & mockable runtime:** No test should require a networked stack. Provide a deterministic TestRuntime with configurable AppSpec.  
* **No hidden magic:** Minimal macros; derives allowed for ergonomics (Schema derive) but not to hide semantics.

Constraints: MontRS should not require specific third-party services. Where third-party crates are helpful (e.g. Governor for rate-limiting), expose them as pluggable backends rather than hard dependencies.

# **3\. High-level architecture**

MontRS splits responsibilities across these layers:

* **Core runtime:** signal scheduler, router resolver, loader/action dispatcher, AppSpec loader.  
* **Reactive primitives:** `Signal<T>` with explicit `mutate(...)` API.  
* **Router & routes:** file-based nested routing, typed path/query params, loader/action attributes.  
* **Modules:** trait-driven feature modules with explicit dependency declarations and init hooks.  
* **AppSpec:** typed, target-aware initializer that composes modules, env, initial state, features, and segments.  
* **ORM layer:** trait-based `DbBackend` supporting `Postgres`, `Sqlite`, and generic `Sql`.  
* **Environment:** typed `EnvConfig` trait and secure secret handling, mockable in TestRuntime.  
* **Rate limiting & feature flags:** trait-driven interfaces with deterministic evaluation.  
* **Test runtime:** `TestRuntime` that boots AppSpec deterministically and provides in-memory resources for tests.  
* **Tooling & scaffold:** `create-mont-app` crate provides project scaffolding and integrates `cargo-make`, `trunk`, `tailwindcss`, `RustUI`, and `axum` in the template.

# **4\. Core runtime APIs (conceptual)**

Below are the canonical trait and type signatures that should be implemented in v0.1. These are intentionally minimal and typed.

## **4.1 Signals (reactivity)**

pub struct Signal\<T\> { /\* private \*/ }

impl\<T\> Signal\<T\> {  
    pub fn new(val: T) \-\> Self;  
    pub fn get(\&self) \-\> \&T;  
    pub fn set(\&self, val: T);  
    // Explicit in-place mutation \-- not a raw \&mut, but a controlled mutate that notifies dependents  
    pub fn mutate\<F: FnOnce(\&mut T)\>(\&self, f: F);  
}

Properties:

* `mutate` enforces capability boundaries; mutation always triggers dependency notification.  
* Signals are usable server-side during SSR and in WASM.

## **4.2 Loader & Action primitives (file-based routing)**

Routes define two kinds of functions:

\#\[loader\]  
fn users\_index(ctx: LoaderCtx) \-\> Result\<LoaderResponse, AppError\> { ... }

\#\[action\]  
fn create\_user(input: CreateUser, ctx: ActionCtx) \-\> Result\<ActionResponse, ActionError\> { ... }

* Loaders are read-only and can return reactive resources to hydrate client-side.  
* Actions are write boundaries mapped to HTTP verbs and form handling.  
* Both have typed error enums.

## **4.3 Module trait**

pub trait Module\<C: AppConfig\> {  
    const NAME: &'static str;  
    type Storage: DbBackend;  
    type Config: Default \+ Clone;  
    type Error: std::error::Error \+ Send \+ Sync \+ 'static;

    // Called during runtime/AppSpec initialization  
    fn init(\&self, ctx: \&mut ModuleContext\<C\>) \-\> Result\<(), Self::Error\>;

    // Optional: expose loaders/actions via registration helpers  
    fn register\_routes(\&self, registrar: \&mut RouteRegistrar\<C\>);  
}

Modules must be independently testable and bootable in TestRuntime.

## **4.4 AppSpec & AppConfig**

pub enum Target { Server, Wasm, Edge, Desktop, MobileAndroid, MobileIos }

pub struct AppSpec\<C: AppConfig\> {  
    pub config: C,  
    pub modules: Vec\<Box\<dyn Module\<C\>\>\>,  
    pub env: TypedEnv,  
    pub initial\_state: serde\_json::Value, // or typed InitialState associated type  
    pub features: FeatureFlags,  
    pub segments: Vec\<Segment\>,  
    pub target: Target,  
}

pub trait AppConfig: Sized \+ 'static {  
    type Db: DbBackend;  
    type Env: EnvConfig;  
    type Error: std::error::Error \+ Send \+ Sync;  
    // other associated types for renderer, auth, etc.  
}

AppSpec is the deterministic artifact used for runtime initialization and TestRuntime bootstrapping.

## **4.5 EnvConfig**

pub trait EnvConfig {  
    fn get\<T: FromEnv\>(\&self, key: \&str) \-\> Result\<T, EnvError\>;  
}

`FromEnv` provides typed conversions and validation. TestRuntime injects a fake `EnvConfig`.

## **4.6 DbBackend (ORM foundation)**

pub trait DbBackend: Send \+ Sync \+ 'static {  
    type Connection;  
    type Error;

    fn acquire(\&self) \-\> Result\<Self::Connection, Self::Error\>;  
    fn execute(\&self, conn: \&mut Self::Connection, sql: \&str, params: &\[\&dyn ToSql\]) \-\> Result\<u64, Self::Error\>;  
    fn query\<T: FromRow\>(\&self, conn: \&mut Self::Connection, sql: \&str, params: &\[\&dyn ToSql\]) \-\> Result\<Vec\<T\>, Self::Error\>;  
}

Implementations: `PostgresBackend`, `SqliteBackend`, and `SqlBackend` (generic).

## **4.7 RateLimiter**

pub trait RateLimiter {  
    fn check(\&self, key: \&str) \-\> Result\<(), RateLimitExceeded\>;  
}

`Governor`\-backed implementation is provided as `GovernorLimiter`, optional dependency.

## **4.8 Feature flags & segments**

pub struct Segment { pub name: String, /\* regions, custom predicate \*/ }  
pub struct FeatureFlags { /\* mapping per target / per segment \*/ }

impl AppSpec {  
    pub fn is\_feature\_enabled(\&self, name: \&str, ctx: \&FeatureCtx) \-\> bool { ... }  
}

# **5\. Schema validation (derive-based, Zod-like ergonomics)**

MontRS exposes a derive macro `#[derive(Schema)]` that generates:

* A typed validation function: `fn validate(&self) -> Result<(), ValidationError>`.  
* Metadata used by forms / CLI to surface validation errors.  
* Integration with loader/action arguments: actions receive validated input types.

Example:

\#\[derive(Schema, Deserialize)\]  
pub struct CreateUser {  
    \#\[schema(min\_len \= 3)\]  
    username: String,

    \#\[schema(email)\]  
    email: String,  
}

Design notes:

* The derive should be lightweight; prefer `serde` \+ a small `montrs-schema` crate that emits validation code, not a runtime schema DSL.  
* Validation errors are typed and included in `ActionError::InvalidInput(ValidationError)`.

# **6\. Module lifecycle & registration**

Modules register routes and declare dependencies:

* `Module::init` is the deterministic init hook called during AppSpec bootstrap.  
* `Module::register_routes` registers loader/action handlers into the router.  
* Modules declare storage needs with an associated `Storage` type to allow mocking.

A module test pattern:

\#\[test\]  
fn user\_module\_create\_and\_list() {  
    let spec \= AppSpec::default().with\_module(UserModule::default()).with\_env(TestEnv::new());  
    let rt \= TestRuntime::new(spec);  
    rt.execute(|| {  
        // call action  
        // call loader  
        // assert DB state  
    });  
}

No networked services required.

# **7\. `TestRuntime` and mocking**

`TestRuntime` boots AppSpec in-process deterministically:

* Supplies in-memory DB or configurable fake `DbBackend`.  
* Supplies `TestEnv` implementing `EnvConfig` with seeded keys/values.  
* Supplies fake clocks/control for `RateLimiter`.  
* Exposes `execute` and `spawn` for sync/async code execution in tests.

Design goal: tests are fast, deterministic, and do not rely on external infra.

# **8\. ORM design (Drizzle-like, minimal abstraction)**

Core design decisions:

* Keep SQL first-class: developers may write SQL strings, with typed `FromRow` derivation.  
* Provide lightweight builder helpers for common operations (insert/update/select) but not a complex ORM.  
* Support Postgres/SQLite via `sqlx`\-style compile-time checked queries as an optional optimization (but not required for v0.1).  
* Make backends pluggable: `DbBackend` trait implementations provided for `postgres`, `sqlite`, and a `Sql` generic layer.  
* Provide in-memory backend for testing.

Example usage:

let mut conn \= db.acquire()?;  
let users: Vec\<User\> \= db.query(\&mut conn, "SELECT id, username FROM users WHERE active \= ?", &\[\&true\])?;  
db.execute(\&mut conn, "INSERT INTO users (username, email) VALUES (?, ?)", &\[\&username, \&email\])?;

Helper functions for non-SQL developers:

db.insert("users", \&user)?; // simple helper, not required

# **9\. Rate limiting & feature flags**

* Rate limiters are pluggable per module/action. The framework supplies `GovernorLimiter` based on the `governor` crate as an optional implementation.  
* Feature flags and segmentation are evaluated during app initialization via AppSpec. Feature availability is deterministic and queryable at runtime.

Example per-action check:

\#\[action\]  
fn comment\_create(input: CreateComment, ctx: ActionCtx) \-\> Result\<(), ActionError\> {  
    ctx.rate\_limiter.check(format\!("create\_comment:{}", ctx.user\_id))?;  
    // continue...  
}

# **10\. `create-mont-app` — CLI scaffold**

## **10.1 Purpose & responsibilities**

`create-mont-app` is a Cargo crate and small binary that scaffolds MontRS projects using a default template. It must:

* Generate a multi-crate workspace (recommended) with:  
  * `montrs-core` crate (runtime primitives)  
  * `montrs-schema` crate (derive macros)  
  * `montrs-orm` crate (DbBackend traits & implementations)  
  * `montrs-cli` (create-mont-app code)  
  * `app/` crate (user application)  
* Populate the AppSpec, example modules, README, docs, CI files, and scripts.  
* Integrate `cargo-make` for common tasks, `trunk` for WASM builds, `tailwindcss` for CSS tooling, `RustUI` for UI abstraction, and `axum` for backend routing in the template app.  
* Create a `docs/` directory with starter API docs and contribution guide.

## **10.2 Command usage (example)**

\# Install  
cargo install create-mont-app

\# Create a new app  
create-mont-app new my-shop \--template default \--license MIT \--vcs git  
cd my-shop  
cargo make init

Default template choices:

* Build tool: `cargo-make`  
* WASM build: `trunk`  
* Styling: `tailwindcss`  
* UI layer: `RustUI` (opinionated set of components)  
* Backend routing: `axum`  
* Task runner config and npm setup are created for tailwind/trunk usage.

## **10.3 Template artifacts**

Template will include:

* `Cargo.toml` workspace  
* `app/` crate (integration with `montrs-core`)  
* `montrs-core/` (reference minimal runtime; used for local dev)  
* `shared-modules/` (example modules: `users`, `auth`, `health`)  
* `examples/` (SSR example, WASM example)  
* `docs/` (see docs section)  
* `Makefile.toml` for `cargo-make` tasks  
* `tailwind.config.js`, `package.json` for minimal tailwind dev build  
* `trunk.toml` and static assets for WASM  
* CI files (.github/workflows) for build/test/deploy

# **11\. Project & repo layout (recommended)**

## **11.1 MontRS framework repo layout (root)**

montrs/  
├─ crates/  
│  ├─ montrs-core/          \# core runtime primitives  
│  ├─ montrs-schema/        \# derive macros for Schema  
│  ├─ montrs-orm/           \# DbBackend trait \+ backends  
│  ├─ montrs-test/          \# TestRuntime & helpers  
│  ├─ montrs-cli/           \# create-mont-app binary  
│  └─ montrs-examples/      \# small example crates  
├─ docs/  
│  ├─ index.md  
│  ├─ architecture.md  
│  ├─ modules.md  
│  ├─ appspec.md  
│  ├─ schema.md  
│  ├─ orm.md  
│  ├─ testing.md  
│  └─ deployment.md  
├─ examples/  
│  ├─ todo/                 \# simple end-to-end example  
│  └─ ecommerce/            \# more advanced example  
├─ scripts/  
│  └─ release.sh  
├─ .github/  
│  └─ workflows/  
│     ├─ ci.yml  
│     └─ publish.yml  
├─ Cargo.toml                \# workspace  
└─ README.md

## **11.2 Template app layout (create-mont-app generated)**

my-app/  
├─ Cargo.toml (workspace)  
├─ Makefile.toml (cargo-make tasks: dev, build, test, lint, fmt)  
├─ trunk.toml  
├─ package.json (tailwind only)  
├─ app/                     \# main application crate (server \+ wasm entry)  
│  ├─ src/  
│  │  ├─ main.rs  
│  │  ├─ routes/  
│  │  │  ├─ index.rs  
│  │  │  ├─ users/  
│  │  │  │  ├─ loader.rs  
│  │  │  │  └─ actions.rs  
│  │  ├─ app-modules/  
│  │  │  ├─ users.rs  
│  │  │  └─ auth.rs  
│  │  ├─ schema/  
│  │  │  └─ user.rs  
│  │  └─ client/           \# WASM client code (RustUI)  
│  ├─ static/  
│  └─ Cargo.toml  
├─ shared-modules/          \# app-specific modules (optional workspace crates)  
├─ docs/  
│  ├─ getting-started.md  
│  ├─ api.md  
│  ├─ contributing.md  
│  └─ deploy.md  
├─ tests/  
│  └─ integration\_tests.rs  
└─ README.md

Template will show best practices: modules isolated, schemas in `schema/`, routes under `routes/`, and a `docs/` folder.

# **12\. Documentation & docs/ hierarchy**

The `docs/` directory must be comprehensive. The framework repo and generated app must both include detailed docs. Suggested structure:

docs/  
├─ index.md                 \# overview and quickstart  
├─ architecture.md          \# deep design rationale  
├─ getting-started.md       \# create-mont-app usage \+ example  
├─ modules.md               \# how to author modules  
├─ appspec.md               \# targeting, segments, features  
├─ router.md                \# file-based routes, loaders/actions  
├─ schema.md                \# derive-based schema usage  
├─ orm.md                   \# DbBackend API & examples  
├─ testing.md               \# TestRuntime, mocking patterns  
├─ deployment.md            \# Cloudflare / Pages / Edge guidance  
├─ cli.md                   \# create-mont-app options & templates  
└─ contributing.md

Docs should include code examples, sequence diagrams where helpful, and a canonical README.md that points to the docs.

# **13\. README.md — canonical contents (both repo and template app)**

README must contain:

* Project purpose & philosophy  
* Quick start (`cargo install create-mont-app` \+ `create-mont-app new`)  
* Example code snippets (boot an AppSpec, register a module)  
* Testing guide (how to run `cargo make test` with TestRuntime)  
* Contributing & code of conduct  
* License & versioning

Include links to `docs/` for deeper reading.

# **14\. Recommended dependencies (non-binding)**

MontRS v0.1 should avoid locking into heavy dependencies. Recommend optionally pluggable crates:

* `tokio` — async runtime (optional for server variant)  
* `axum` — HTTP routing & server (used in template)  
* `trunk` — WASM bundling (template)  
* `tailwindcss` — CSS (via npm in template)  
* `sqlx` or `postgres`/`rusqlite` — DB drivers (pluggable implementations)  
* `serde` — (de)serialization  
* `thiserror` — error derive  
* `governor` — optional rate-limiter backend  
* `anyhow` — prototyping only (avoid in public APIs)  
* `once_cell` / `lazy_static` — small conveniences  
* `wasm-bindgen`, `gloo` — for client-side WASM support (only in client crate)

Note: Keep runtime optional; each capability is behind traits so implementations are pluggable and testable.

# **15\. CI/CD & Release process**

**Semantic Versioning:**

* Use `MAJOR.MINOR.PATCH`. v0.x implies experimental; v1.0 indicates stabilization.  
* Each release must include changelog and migration notes for breaking changes.

**CI pipeline (.github/workflows/ci.yml):**

* Steps:  
  * `cargo fmt -- --check`  
  * `cargo clippy -- -D warnings`  
  * `cargo test --workspace`  
  * `cargo build --workspace --release`  
  * WASM: `trunk build` (for examples)  
  * Lint docs & markdown  
  * Run integration smoke tests under TestRuntime

**Publish pipeline (.github/workflows/publish.yml):**

* Tag-based release using `cargo-release` or manual script `scripts/release.sh`.  
* Publish `create-mont-app` to crates.io on release.  
* Generate release artifacts for examples and template (zip).

**Cloudflare deployment (see section 18\)** includes automated `wrangler` steps for worker deployment and Pages static asset publishing.

# **16\. Security & best practices**

* CSRF protection by default on actions (forms have CSRF tokens surfaced through AppSpec).  
* Typed input validation (Schema) to prevent injection.  
* Secrets never stored in code—`EnvConfig` is the only typed getter; TestRuntime uses stubbed secrets.  
* Rate-limiting defaults for public endpoints; module authors can opt-in or override.

# **17\. Migration & extensibility**

* Modules are versioned: include `Module::version()` and `Module::migrate(prev_version)` hooks for state upgrades.  
* AppSpec can accept versioned initializers to support rolling upgrades in production.

# **18\. Deployment & Cloudflare guidance**

**Cloudflare Workers (Edge)**:

* Build WASM using `trunk` or `wasm-pack`, bundle serverless handler with `wrangler` or Cloudflare's Workers for Rust support.  
* Use `AppSpec` target `Edge` to change features (e.g., smaller ORM, different caching).  
* Provide a `wrangler.toml` in the template, and a `Makefile` target `cargo make deploy:cloudflare` that:  
  * Builds the WASM/worker bundle  
  * Runs `wrangler publish` with a bound KV/Secrets config read from AppSpec.

**Cloudflare Pages (static \+ functions)**:

* `trunk` builds static assets; Pages can serve them.  
* For SSR or API, deploy functions as Workers and static frontend to Pages.  
* CI should run `trunk build` then push static assets to Pages via API or GitHub integration; publish worker via `wrangler` in same pipeline.

**Serverful deployments (optional)**:

* `axum` server crate builds a standard binary for containers.  
* Prefer ephemeral container images with environment variables injected via secrets manager.  
* Provide a sample `Dockerfile` for `app/` crate and `deploy` script for Kubernetes/managed runners.

**Edge caveats**:

* Some DB drivers are not available on edge. Use AppSpec to configure a proxy or use server-side edge function to reach central DB. For pure edge apps, use in-memory or remote API pattern.

# **19\. Testing & quality**

* Unit tests for modules using `TestRuntime`.  
* Integration tests that instantiate AppSpec and exercise multiple modules.  
* Example test layout:

tests/  
├─ unit/  
│  └─ user\_module\_tests.rs  
├─ integration/  
│  └─ e2e.rs  
│  └─ fuzzing.rs

* Use `cargo-make` tasks: `cargo make test` runs all tests including WASM tests and integration tests. Use `cargo test --workspace` in CI.

# **20\. Roadmap to v0.1 (milestones)**

**M0 — Core primitives**

* Implement `Signal<T>`, basic router, loader/action dispatch, Module trait, AppSpec, EnvConfig skeleton.

**M1 — Schema & derive**

* Implement `montrs-schema` derive for basic validators and integration with actions.

**M2 — ORM & DB backends**

* Implement `DbBackend` trait and `SqliteBackend` in-memory for tests; Postgres backend optional.

**M3 — TestRuntime**

* Implement `TestRuntime`, TestEnv, in-memory DB, and module tests for example modules.

**M4 — create-mont-app template**

* CLI scaffold, workspace template, `app/` example, tailwind/trunk integration.

**M5 — Feature flags & rate limiter**

* Implement feature flag evaluation, a Governor-backed limiter, API usage examples.

**M6 — Docs & CI**

* Flesh out docs, example apps, CI pipelines, Cloudflare deploy guide.

Release v0.1 once M4–M6 are complete and integration tests pass.

# **21\. Example snippets & sample flows**

## **21.1 Bootstrapping AppSpec**

let spec \= AppSpec::\<MyConfig\>::new()  
    .with\_module(Box::new(UserModule::default()))  
    .with\_env(MyEnv::from\_vars())  
    .with\_target(Target::Server);

let runtime \= Runtime::new(spec)?;  
runtime.serve(); // axum or worker-bridge

## **21.2 Simple loader**

\#\[loader\]  
async fn user\_loader(ctx: LoaderCtx, id: UserId) \-\> Result\<UserDto, ActionError\> {  
    let db \= ctx.db();  
    let mut conn \= db.acquire()?;  
    let users \= db.query::\<User\>(\&mut conn, "SELECT id, username FROM users WHERE id \= ?", &\[\&id\])?;  
    users.into\_iter().next().ok\_or(ActionError::NotFound)  
}

## **21.3 Schema & action**

\#\[derive(Schema, Deserialize)\]  
pub struct CreateUser {  
    \#\[schema(min\_len \= 3)\]  
    username: String,  
    \#\[schema(email)\]  
    email: String,  
}

\#\[action\]  
fn create\_user(input: CreateUser, ctx: ActionCtx) \-\> Result\<JsonResponse, ActionError\> {  
    input.validate()?;  
    // store user...  
}

# **22\. README & CONTRIBUTING (brief)**

* Provide contributor guidelines, code formatting, license (MIT), code of conduct, and PR process.  
* Enforce `rustfmt`, `clippy` in CI.  
* Encourage small, modular PRs and include module tests.

# **23\. Licensing & governance**

* Default repo license: **MIT**.  
* Use semantic versioning (MAJOR.MINOR.PATCH).  
* For governance, adopt a simple MAINTAINERS.md file listing core maintainers and code ownership for Afsall Labs.

# **24\. Deliverables from PRD to implementation**

To go from PRD → v0.1 implementable artifact, deliver:

1. `montrs-core` crate implementing runtime primitives and signals.  
2. `montrs-schema` derive crate with basic validators.  
3. `montrs-orm` crate with DbBackend trait and sqlite \+ in-memory backends.  
4. `montrs-test` crate with TestRuntime and TestEnv.  
5. `montrs-cli` crate implementing `create-mont-app`.  
6. Workspace templates and `examples/` demonstrating server+wasm usage.  
7. `docs/` content for all public APIs and developer guides.  
8. CI configs and `scripts/release.sh` to perform semantic versioned releases.

# **25\. Appendices**

## **A. “Do’s and Don’ts”**

**Do**

* Favor explicit trait contracts.  
* Keep modules small & composable.  
* Keep SQL explicit and accessible.  
* Make things mockable for deterministic tests.

**Don’t**

* Avoid global singletons for app state.  
* Avoid runtime reflection or magic inferred behavior.  
* Avoid abstracting SQL away into hidden DSLs.

## **B. Glossary (short)**

* **AppSpec** — typed initializer & deployment spec.  
* **Module** — composable feature unit (analogous to Substrate pallet).  
* **Signal** — reactive primitive.  
* **Loader** — read-only route function.  
* **Action** — write/mutation route function.

# **26\. Conclusion**

MontRS unifies best-in-class design patterns in a Rust-native framework: Leptos-style reactivity, Remix-style routing mental model, Dioxus ergonomics, Yew discipline, Substrate modularity & testability, and Drizzle minimal ORM abstraction. The `create-mont-app` CLI and template will accelerate adoption and produce opinionated, production-capable projects using `cargo-make`, `trunk`, `tailwindcss`, `RustUI`, and `axum`. The deterministic AppSpec plus TestRuntime ensures reproducible tests and safe deployments to Cloudflare Workers / Pages and other targets.

This PRD defines an actionable route to MontRS v0.1: small set of core crates, clear trait interfaces, a deterministic spec, minimal-DSL schema derives, a Drizzle-like ORM, and an approachable developer experience with strong test invariants. Follow the roadmap and implementation deliverables, and MontRS will be positioned to provide a Rust-first web framework that scales from prototypes to production systems while preserving correctness, performance, and developer ergonomics.

