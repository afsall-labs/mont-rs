// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
// This file is part of montrs.
// Copyright (C) 2026-Present Afsall Inc.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// Alternatively, this file is available under the MIT License:
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! montrs-metadata: Project metadata abstraction for MontRS.
//!
//! Reads `montrs.toml` and provides all configuration needed for building,
//! serving, and deploying MontRS applications.
//!
//! # Example `montrs.toml`
//! ```toml
//! [project]
//! name = "my-app"
//!
//! [serve]
//! site-addr = "0.0.0.0:3000"
//! tailwind-input-file = "style/main.css"
//! site-root = "target/site"
//! site-pkg-dir = "pkg"
//! package = "app"
//! ```
//!
//! # Extended Schema (Phase 1+)
//! - `[deploy]` — deployment mode (ssr, static, desktop, mobile)
//! - `[env]` — environment variables
//! - `[settings]` — all settings (no separate settings.toml)
//! - `[monorepo]` — monorepo workspace config
//! - `[tools]` — tool version definitions (Phase 3)
//! - `[deps]` — dependency metadata (Phase 3)
//! - `[aliases]` — tool/version aliases (Phase 3)
//! - `[services]` — daemon/service definitions (Phase 4)
//! - `[proxy]` — reverse proxy config (Phase 4)

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The full MontRS project metadata, read from `montrs.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MontrsMetadata {
    #[serde(default)]
    pub project: ProjectMeta,
    #[serde(default)]
    pub serve: ServeMeta,
    #[serde(default)]
    pub build: BuildMeta,
    #[serde(default)]
    pub deploy: DeployMeta,
    #[serde(default)]
    pub env: EnvSection,
    #[serde(default)]
    pub settings: SettingsSection,
    #[serde(default)]
    pub monorepo: MonorepoSection,
    #[serde(default)]
    pub tools: std::collections::HashMap<String, toml::Value>,
    #[serde(default)]
    pub deps: std::collections::HashMap<String, toml::Value>,
    #[serde(default)]
    pub aliases: std::collections::HashMap<String, toml::Value>,
    #[serde(default)]
    pub services: std::collections::HashMap<String, toml::Value>,
    #[serde(default)]
    pub proxy: std::collections::HashMap<String, toml::Value>,
    #[serde(default)]
    pub tasks: std::collections::HashMap<String, toml::Value>,
}

/// Project identity metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectMeta {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

/// Serve/build configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ServeMeta {
    /// The single package name for both WASM and SSR.
    #[serde(default)]
    pub package: Option<String>,
    /// Output name for the WASM binary.
    #[serde(default)]
    pub output_name: Option<String>,
    /// Site address (default: "0.0.0.0:3000").
    #[serde(default = "default_site_addr")]
    pub site_addr: String,
    /// Port for the live reload WebSocket (default: 3001).
    #[serde(default = "default_reload_port")]
    pub reload_port: u16,
    /// Path to the site root directory (default: "target/site").
    #[serde(default = "default_site_root")]
    pub site_root: String,
    /// Path to the WASM package directory (default: "pkg").
    #[serde(default = "default_site_pkg_dir")]
    pub site_pkg_dir: String,
    /// Path to the Tailwind CSS input file.
    pub tailwind_input_file: Option<String>,
    /// Directory for static assets.
    pub assets_dir: Option<String>,
    /// Browser compatibility query (default: "defaults").
    #[serde(default = "default_browserquery")]
    pub browserquery: String,
    /// Features to enable for the WASM library.
    #[serde(default)]
    pub lib_features: Vec<String>,
    /// Whether to use default features for the WASM library.
    #[serde(default = "default_true")]
    pub lib_default_features: bool,
    /// Features to enable for the server binary.
    #[serde(default)]
    pub bin_features: Vec<String>,
    /// Whether to use default features for the server binary.
    #[serde(default = "default_true")]
    pub bin_default_features: bool,
    /// Build the site with `--release` (optimized). Recommended for anything
    /// other than quick local iteration: release builds of the SSR server and
    /// the WASM client are far smaller and faster, and skip the runtime
    /// "outside a reactive tracking context" diagnostics that only fire in
    /// debug builds.
    #[serde(default)]
    pub release: bool,
    /// Whether to hash frontend files.
    #[serde(default)]
    pub hash_files: bool,
    /// Additional files to watch for changes.
    #[serde(default)]
    pub watch_additional_files: Vec<String>,
    /// Path to the style file.
    pub style_file: Option<String>,
}

impl Default for ServeMeta {
    fn default() -> Self {
        Self {
            package: None,
            output_name: None,
            site_addr: default_site_addr(),
            reload_port: default_reload_port(),
            site_root: default_site_root(),
            site_pkg_dir: default_site_pkg_dir(),
            tailwind_input_file: None,
            assets_dir: None,
            browserquery: default_browserquery(),
            lib_features: Vec::new(),
            lib_default_features: true,
            bin_features: Vec::new(),
            bin_default_features: true,
            release: false,
            hash_files: false,
            watch_additional_files: Vec::new(),
            style_file: None,
        }
    }
}

/// Build configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuildMeta {
    #[serde(default)]
    pub release: bool,
    #[serde(default)]
    pub target: String,
}

impl Default for BuildMeta {
    fn default() -> Self {
        Self {
            release: false,
            target: "index.html".to_string(),
        }
    }
}

/// Deployment mode configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeployMeta {
    /// Deployment mode: "ssr" | "static" | "desktop" | "mobile"
    #[serde(default = "default_deploy_mode")]
    pub mode: String,
}

impl Default for DeployMeta {
    fn default() -> Self {
        Self {
            mode: default_deploy_mode(),
        }
    }
}

fn default_deploy_mode() -> String {
    "ssr".to_string()
}

/// Environment variable section in montrs.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvSection {
    /// Environment variables as key-value pairs.
    #[serde(default, flatten)]
    pub vars: std::collections::HashMap<String, toml::Value>,
}

/// Settings section (replaces standalone settings.toml).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsSection {
    #[serde(default, flatten)]
    pub values: std::collections::HashMap<String, toml::Value>,
}

/// Monorepo configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct MonorepoSection {
    /// Directories that are workspace members.
    #[serde(default)]
    pub members: Vec<String>,
    /// Whether to auto-discover workspace members.
    #[serde(default)]
    pub auto_discover: bool,
}

fn default_site_addr() -> String {
    "0.0.0.0:3000".to_string()
}

fn default_reload_port() -> u16 {
    3001
}

fn default_site_root() -> String {
    "target/site".to_string()
}

fn default_site_pkg_dir() -> String {
    "pkg".to_string()
}

fn default_browserquery() -> String {
    "defaults".to_string()
}

fn default_true() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Lightweight Cargo workspace discovery.
//
// Replaces `cargo_metadata` (a subprocess) for the two things `montrs serve`
// needs: the root package name, and finding the workspace member that is the
// "serve package" (declares both a `cdylib` lib and a `[[bin]]`).
// ---------------------------------------------------------------------------

/// Minimal description of a Cargo package used for workspace discovery.
struct CargoPackage {
    name: String,
    has_cdylib: bool,
    has_bin: bool,
}

/// Parse `[package]` name and target hints from a `Cargo.toml` file.
fn parse_cargo_package(manifest: &Path) -> Option<CargoPackage> {
    let content = std::fs::read_to_string(manifest).ok()?;
    let value: toml::Value = toml::from_str(&content).ok()?;
    let name = value.get("package")?.get("name")?.as_str()?.to_string();
    let has_cdylib = value
        .get("lib")
        .and_then(|l| l.get("crate-type"))
        .and_then(|ct| ct.as_array())
        .map(|a| a.iter().any(|v| v.as_str() == Some("cdylib")))
        .unwrap_or(false);
    // A package has a binary target if it declares `[[bin]]` or has
    // `src/main.rs` (Cargo auto-discovers the latter, matching what
    // cargo_metadata's `targets` would report).
    let has_bin = value.get("bin").is_some()
        || manifest
            .parent()
            .map(|p| p.join("src/main.rs").exists())
            .unwrap_or(false);
    Some(CargoPackage {
        name,
        has_cdylib,
        has_bin,
    })
}

/// Expand a workspace member pattern (which may contain a single `*`) into
/// concrete directories under `root`.
fn expand_member_pattern(root: &Path, pattern: &str) -> Vec<PathBuf> {
    let full = root.join(pattern);
    let s = full.to_string_lossy();
    let Some((prefix, suffix)) = s.split_once('*') else {
        return if full.is_dir() {
            vec![full]
        } else {
            Vec::new()
        };
    };
    let parent = Path::new(prefix).parent().unwrap_or(Path::new(prefix));
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let candidate = PathBuf::from(format!("{prefix}{name}{suffix}"));
            if candidate.is_dir() {
                out.push(candidate);
            }
        }
    }
    out
}

/// Read workspace member directories from `[workspace] members`,
/// `default-members`, and `exclude` in the root `Cargo.toml`.
fn workspace_member_dirs(root: &Path) -> Vec<PathBuf> {
    let Ok(content) = std::fs::read_to_string(root.join("Cargo.toml")) else {
        eprintln!(
            "[montrs-metadata] workspace: read Cargo.toml FAILED at {:?}",
            root.join("Cargo.toml")
        );
        return Vec::new();
    };
    let Ok(value) = toml::from_str::<toml::Value>(&content) else {
        return Vec::new();
    };
    let Some(workspace) = value.get("workspace") else {
        return Vec::new();
    };

    let patterns = |key: &str| -> Vec<String> {
        workspace
            .get(key)
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|m| m.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    };

    let members = patterns("members");
    let default_members = patterns("default-members");
    let excludes = patterns("exclude");

    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for pat in members.iter().chain(default_members.iter()) {
        for dir in expand_member_pattern(root, pat) {
            if seen.insert(dir.clone()) {
                dirs.push(dir);
            }
        }
    }

    // Drop any member that falls under an excluded pattern.
    let excluded: Vec<PathBuf> = excludes
        .iter()
        .flat_map(|pat| expand_member_pattern(root, pat))
        .collect();
    dirs.retain(|dir| !excluded.iter().any(|ex| dir.starts_with(ex)));
    dirs
}

/// Discover all Cargo packages (root + workspace members) under `root`.
fn discover_packages(root: &Path) -> Vec<CargoPackage> {
    let mut packages = Vec::new();
    if let Some(pkg) = parse_cargo_package(&root.join("Cargo.toml")) {
        packages.push(pkg);
    }
    for dir in workspace_member_dirs(root) {
        if let Some(pkg) = parse_cargo_package(&dir.join("Cargo.toml")) {
            packages.push(pkg);
        }
    }
    packages
}

impl MontrsMetadata {
    /// Load metadata from a `montrs.toml` file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let mut meta: Self = toml::from_str(&content)?;

        let project_path = path
            .as_ref()
            .canonicalize()
            .unwrap_or_default()
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();

        // Auto-detect project name from the root Cargo.toml if not set.
        if meta.project.name.is_none()
            && let Some(root_pkg) =
                parse_cargo_package(&project_path.join("Cargo.toml"))
        {
            meta.project.name = Some(root_pkg.name);
        }

        // If `package` is set, use it for both bin and lib discovery;
        // otherwise auto-discover from workspace members.
        let pkg_name = meta.serve.package.clone();

        for package in discover_packages(&project_path) {
            if let Some(ref name) = pkg_name {
                if package.name == *name {
                    meta.serve.package = Some(package.name.clone());
                    break;
                }
            } else if package.has_cdylib && package.has_bin {
                meta.serve.package = Some(package.name.clone());
                break;
            }
        }

        Ok(meta)
    }
}
