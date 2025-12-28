# todo-example

A practical demonstration of the MontRS framework in action.

## Overview

This example showcases how to combine the various crates within the MontRS ecosystem to build a real-world application. It covers everything from state management to database persistence.

## Features Demonstrated

- **Signal Reactivity**: Using the `Signal` type for thread-safe, reactive state.
- **Schema Validation**: Using `#[derive(Schema)]` to enforce data constraints (like minimum title length).
- **ORM Integration**: Performing asynchronous database operations using `SqliteBackend`.
- **Modularity**: Structuring the application using the `Module` trait and `AppSpec`.

## Running the Example

From the root of the workspace, run:

```bash
cargo run -p todo-example
```

## Code Highlights

- **`CreateTodo`**: Demonstrates field-level validation attributes.
- **`Todo`**: Shows how to implement `FromRow` for different database backends.
- **`TodoModule`**: Illustrates the unit of composition in a MontRS app.
