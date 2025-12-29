//! montrs-cli: The official scaffolding tool for MontRS.
//! This tool helps developers initialize new projects with a workspace-first
//! architecture and integration with common developer tools.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

/// Command-line arguments for the CLI tool.
#[derive(Parser)]
#[command(name = "create-mont-app")]
#[command(about = "Scaffold a new MontRS application", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new project from a template.
    New {
        /// Name of the project to be created as a subdirectory.
        name: String,
        /// Template to use for scaffolding (default is currently the only supported template).
        #[arg(short, long, default_value = "default")]
        template: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New { name, template } => {
            provision_project(name, template).await?;
        }
    }

    Ok(())
}

/// Core logic for provisioning a new MontRS project.
/// Handles directory creation and template files generation.
async fn provision_project(name: &str, _template: &str) -> anyhow::Result<()> {
    println!("{} Creating new project: {}", style("🚀").bold(), style(name).cyan().bold());
    
    let base_path = PathBuf::from(name);
    if base_path.exists() {
        return Err(anyhow::anyhow!("Directory {} already exists", name));
    }

    // Set up a progress bar for visual feedback during project creation.
    let pb = ProgressBar::new(4);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")?
        .progress_chars("#>-"));

    // 1. Initialize the directory structure (workspace-first).
    pb.set_message("Creating directory structure...");
    fs::create_dir_all(base_path.join("crates"))?;
    fs::create_dir_all(base_path.join("app/src"))?;
    fs::create_dir_all(base_path.join("docs"))?;
    pb.inc(1);

    // 2. Write the root workspace Cargo.toml.
    pb.set_message("Writing workspace config...");
    let cargo_toml = format!(r#"[workspace]
resolver = "2"
members = ["app", "crates/*"]

[workspace.package]
version = "0.1.0"
edition = "2024"
"#);
    fs::write(base_path.join("Cargo.toml"), cargo_toml)?;
    pb.inc(1);

    // 3. Generate the application core skeleton.
    pb.set_message("Generating application core...");
    let app_cargo = r#"[package]
name = "app"
version = "0.1.0"
edition = "2024"

[dependencies]
montrs-core = { git = "https://github.com/afsall-labs/mont-rs.git" }
"#;
    fs::write(base_path.join("app/Cargo.toml"), app_cargo)?;
    fs::write(base_path.join("app/src/main.rs"), "fn main() { println!(\"Hello, MontRS!\"); }")?;
    pb.inc(1);

    // 4. Add developer ergonomics (Makefiles, Trunk, etc.).
    pb.set_message("Adding developer ergonomics...");
    let makefile = r#"[tasks.dev]
command = "cargo"
args = ["run", "-p", "app"]

[tasks.test]
command = "cargo"
args = ["test", "--workspace"]

[tasks.build]
command = "cargo"
args = ["build", "--release"]
"#;
    fs::write(base_path.join("Makefile.toml"), makefile)?;

    let trunk_toml = r#"[build]
target = "index.html"
dist = "dist"

[serve]
port = 8080
"#;
    fs::write(base_path.join("trunk.toml"), trunk_toml)?;

    let gitignore = r#"/target
/dist
**/*.rs.bk
Cargo.lock
.env
"#;
    fs::write(base_path.join(".gitignore"), gitignore)?;
    pb.inc(1);

    // 5. Finalize setup.
    pb.set_message("Finalizing...");
    pb.inc(1);
    pb.finish_with_message("Done!");

    println!("\n{} Project {} created successfully!", style("✨").green().bold(), style(name).cyan().bold());
    println!("Next steps:\n  cd {}\n  cargo run -p app", name);

    Ok(())
}
