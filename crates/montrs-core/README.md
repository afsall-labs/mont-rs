# montrs-core

The core runtime and architectural backbone of the MontRS framework.

## Overview

`montrs-core` provides the fundamental building blocks for building deterministic, reactive Rust applications. It defines the application lifecycle, routing model, and state management primitives.

## Key Components

- **Signals**: Atomic, thread-safe reactivity system for fine-grained state tracking.
- **Module System**: Trait-driven composition model (`Module` and `AppSpec`).
- **Router**: Explicit, async-first routing with `Loaders` and `Actions`.
- **Typed Env**: Safe and mockable environment variable management.
- **Rate Limiting**: Integrated request throttling via the `Limiter` trait.
- **Feature Flags**: Dynamic feature toggles and user segmentation.

## Usage

```rust
use montrs_core::Signal;

let counter = Signal::new(0);
let val = counter.get();
counter.set(val + 1);
```
