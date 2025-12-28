# montrs-test

Deterministic testing utilities for the MontRS framework.

## Overview

`montrs-test` provides the infrastructure needed to write robust unit and integration tests for MontRS applications. It allows you to mock environment variables and execute application logic in a controlled, in-process runtime.

## Key Features

- **TestEnv**: A mockable environment provider that allows you to programmatically set variables.
- **TestRuntime**: A wrapper around `AppSpec` that facilitates running tests against your application's configuration and modules.

## Usage

```rust
use montrs_test::{TestEnv, TestRuntime};

let env = TestEnv::new();
env.set("DATABASE_URL", "sqlite::memory:");

let spec = AppSpec::new(config, env);
let runtime = TestRuntime::new(spec);

runtime.execute(|spec| {
    // Assert against spec state
}).await;
```
