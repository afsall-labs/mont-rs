# montrs-schema

Procedural macros for schema validation in the MontRS framework.

## Overview

`montrs-schema` provides a powerful `#[derive(Schema)]` macro that enables declarative validation of data structures at compile-time. It generates a `validate()` method for your structs based on schema attributes.

## Supported Attributes

- `#[schema(min_len = N)]`: Ensures a string field has at least `N` characters.
- `#[schema(email)]`: Performs a basic email format check.
- `#[schema(regex = "pattern")]`: Skeleton for regular expression matching.
- `#[schema(custom = "method_name")]`: Delegates validation to a custom method on the struct.

## Usage

```rust
use montrs_schema::Schema;

#[derive(Schema)]
struct UserRegistration {
    #[schema(min_len = 3)]
    username: String,
    #[schema(email)]
    email: String,
}

let reg = UserRegistration { ... };
reg.validate()?;
```
