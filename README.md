<!-- MontRS: The Rust-native, trait-driven deterministic web framework. -->
<!-- This README provides a high-level overview of the framework and its components. -->

# MontRS

**MontRS** is a Rust-native, trait-driven, deterministic web framework. It blends the engineering strengths of Leptos (fine-grained reactivity), Remix (web-native routing), Dioxus (multi-target ergonomics), Yew (architecture discipline), Substrate (trait-first modularity), and Drizzle (minimal ORM).

## 🚀 Philosophy

- **Compile-time correctness:** Type-driven design; traits and typed configs everywhere.
- **Deterministic execution:** AppSpec is the single source of truth for initialization.
- **Explicit boundaries:** Loaders for reads, Actions for writes.
- **Minimal abstraction:** SQL-first ORM; no hidden magic.

## 📦 Getting Started

### Install the CLI
```bash
cargo install --path crates/montrs-cli
```

### Create a new app
```bash
create-mont-app new my-awesome-app
cd my-awesome-app
cargo run -p app
```

## 🛠 Project Structure

- `crates/montrs-core`: Core runtime, signals, and routing.
- `crates/montrs-schema`: Validation macros.
- `crates/montrs-orm`: DbBackend traits and SQLite implementation.
- `crates/montrs-test`: TestRuntime for deterministic testing.
- `crates/montrs-cli`: Scaffolding tool.

## 📖 Example: Reactive Counter

```rust
use montrs_core::Signal;

let counter = Signal::new(0);
println!("Value: {}", counter.get());

counter.set(10); // Notifies all subscribers
```

## 📄 License

This project is licensed under the MIT License.
