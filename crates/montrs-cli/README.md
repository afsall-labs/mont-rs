# montrs-cli

The official command-line scaffolding tool for the MontRS framework.

## Overview

`create-mont-app` is a CLI tool designed to jumpstart MontRS development by scaffolding a full workspace-first project structure with sensible defaults and developer tool integrations.

## Key Features

- **Workspace Integration**: Automatically sets up a Cargo workspace for modular development.
- **Developer Ergonomics**: Includes pre-configured `Makefile.toml` for `cargo-make` and `trunk.toml` for web development.
- **Scaffolding**: Generates application skeletons, documentation folders, and git ignore files.

## Installation

```bash
cargo install --path crates/montrs-cli
```

## Usage

```bash
create-mont-app new my-awesome-project
cd my-awesome-project
cargo run -p app
```
