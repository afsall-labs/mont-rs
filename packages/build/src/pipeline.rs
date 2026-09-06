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

use crate::{copy_dir, run_cargo, run_tailwind};
use anyhow::{Result, anyhow};
use montrs_build_core::{BuildPipeline, find_workspace_target_dir};
use montrs_metadata::MontrsMetadata;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// The MontRS build pipeline.
pub struct Pipeline {
    pub meta: MontrsMetadata,
    pub project_root: PathBuf,
    pub site_root: PathBuf,
    pub pkg_dir: PathBuf,
    pub server_bin_name: String,
    pub workspace_target_dir: PathBuf,
    /// Whether to build optimized (--release) artifacts.
    pub release: bool,
    /// Path to the Tailwind CSS binary (managed install override).
    pub tailwind_bin: Option<PathBuf>,
    /// Path to the wasm-bindgen binary (managed install override).
    pub wasm_bindgen_bin: Option<PathBuf>,
}

impl Pipeline {
    pub fn from_root(root: &Path) -> Result<Self> {
        let root = root.canonicalize()?;
        let meta = MontrsMetadata::from_file(root.join("montrs.toml"))?;
        let site_root = root.join(&meta.serve.site_root);
        let pkg_dir = site_root.join(&meta.serve.site_pkg_dir);
        let workspace_target = find_workspace_target_dir(&root)?;
        let release = meta.serve.release;
        let server_bin_name = meta
            .serve
            .package
            .as_deref()
            .unwrap_or("app")
            .replace('-', "_")
            + "-ssr";

        Ok(Self {
            meta,
            project_root: root.to_path_buf(),
            site_root,
            pkg_dir,
            server_bin_name,
            workspace_target_dir: workspace_target,
            release,
            tailwind_bin: None,
            wasm_bindgen_bin: None,
        })
    }

    /// Directory that cargo builds artifacts into for the current profile.
    fn profile_dir(&self) -> &'static str {
        if self.release { "release" } else { "debug" }
    }

    /// Path to the compiled SSR server binary, with `.exe` on Windows.
    pub fn server_bin_path(&self) -> PathBuf {
        let mut name = self.server_bin_name.clone();
        if cfg!(windows) && !name.ends_with(".exe") {
            name.push_str(".exe");
        }
        self.workspace_target_dir
            .join(self.profile_dir())
            .join(name)
    }

    /// Args for `cargo build` of the SSR server binary.
    ///
    /// Order matters: the `--features` value must immediately follow the
    /// `--features` flag, other flags come after.
    pub fn server_args(&self) -> Vec<String> {
        server_build_args(
            self.meta.serve.package.as_deref().unwrap_or("app"),
            &self.meta.serve.bin_features,
            self.meta.serve.bin_default_features,
            self.release,
        )
    }

    fn build_frontend_args(&self) -> Vec<String> {
        let pkg = self.meta.serve.package.as_deref().unwrap_or("app");
        frontend_build_args(
            pkg,
            &self.meta.serve.lib_features,
            self.meta.serve.lib_default_features,
        )
    }

    fn bundle_wasm(&self) -> Result<()> {
        std::fs::create_dir_all(&self.pkg_dir)?;

        let lib_name = self
            .meta
            .serve
            .package
            .as_deref()
            .unwrap_or("app")
            .replace('-', "_");

        let wasm_target_dir = self
            .workspace_target_dir
            .join("wasm32-unknown-unknown")
            // The frontend build always passes `--release` (see
            // build_frontend_args), so the WASM output is always in `release/`.
            .join("release");

        let wasm_file = wasm_target_dir.join(format!("{}.wasm", lib_name));

        if !wasm_file.exists() {
            return Err(anyhow!(
                "WASM file not found at {}. Did the wasm32-unknown-unknown \
                 build succeed?",
                wasm_file.display()
            ));
        }

        let mut cmd = match &self.wasm_bindgen_bin {
            Some(path) => Command::new(path),
            None => Command::new("wasm-bindgen"),
        };
        let status = cmd
            .arg("--target")
            .arg("web")
            .arg("--no-typescript")
            .arg("--out-dir")
            .arg(&self.pkg_dir)
            .arg("--out-name")
            .arg("front")
            .arg(&wasm_file)
            .status();

        // `--out-name front` produces `front.js` + `front_bg.wasm`. Remove any
        // stale `front.wasm` (a leftover from the raw fallback copy) so the
        // browser never downloads the giant unprocessed build.
        let stale = self.pkg_dir.join("front.wasm");
        if stale.exists() {
            let _ = std::fs::remove_file(&stale);
        }

        match status {
            Ok(s) if s.success() => {
                println!(" wasm-bindgen completed successfully");
            }
            Ok(_) => {
                println!(" wasm-bindgen failed — falling back to manual copy");
                self.fallback_copy_wasm(&wasm_file, &lib_name)?;
            }
            Err(_e) => {
                println!(
                    " wasm-bindgen not found — falling back to manual copy"
                );
                self.fallback_copy_wasm(&wasm_file, &lib_name)?;
            }
        }

        Ok(())
    }

    fn fallback_copy_wasm(
        &self,
        wasm_file: &Path,
        lib_name: &str,
    ) -> Result<()> {
        // Match the name wasm-bindgen would produce (`front_bg.wasm`) so the
        // generated index.html points at the same file either way.
        std::fs::copy(wasm_file, self.pkg_dir.join("front_bg.wasm"))?;
        let wasm_target_dir = self
            .workspace_target_dir
            .join("wasm32-unknown-unknown")
            // Frontend always builds with --release.
            .join("release");
        let js_bindings = wasm_target_dir.join(format!("{}.js", lib_name));
        if js_bindings.exists() {
            std::fs::copy(&js_bindings, self.pkg_dir.join("front.js"))?;
        }
        Ok(())
    }
}

impl BuildPipeline for Pipeline {
    fn build_server(&self) -> Result<()> {
        println!(" Building SSR server...");
        run_cargo(&self.server_args())?;
        println!(" SSR server built successfully");
        Ok(())
    }

    fn build_frontend(&self) -> Result<()> {
        println!(" Building frontend (WASM)...");
        run_cargo(&self.build_frontend_args())?;
        println!(" Bundling WASM with wasm-bindgen...");
        self.bundle_wasm()?;
        println!(" Frontend built successfully");
        Ok(())
    }

    fn process_tailwind(&self) -> Result<()> {
        if let Some(tw_input) = &self.meta.serve.tailwind_input_file {
            let input = self.project_root.join(tw_input);
            let output = self.site_root.join("main.css");
            if input.exists() {
                println!(" Processing Tailwind CSS...");
                std::fs::create_dir_all(&self.site_root)?;
                run_tailwind(self.tailwind_bin.as_deref(), &input, &output)?;
                println!(" Tailwind CSS processed");
            }
        }
        Ok(())
    }

    fn copy_assets(&self) -> Result<()> {
        if let Some(assets) = &self.meta.serve.assets_dir {
            let src = self.project_root.join(assets);
            if src.exists() {
                println!(" Copying assets...");
                copy_dir(&src, &self.site_root)?;
                println!(" Assets copied");
            }
        }
        Ok(())
    }

    fn generate_index_html(&self) -> Result<()> {
        let index_path = self.site_root.join("index.html");
        let project_name =
            self.meta.project.name.as_deref().unwrap_or("MontRS App");

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{project_name}</title>
    <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
    <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32.png" />
    <link rel="apple-touch-icon" href="/favicon-180.png" />
    <link rel="stylesheet" href="/main.css" />
    <link rel="modulepreload" href="/pkg/front.js" />
    <script type="module">
        import init, {{ hydrate }} from '/pkg/front.js';
        init('/pkg/front_bg.wasm').then(() => hydrate());
    </script>
</head>
<body>
    <div id="app"></div>
</body>
</html>"#,
        );
        std::fs::write(&index_path, html)?;
        println!(" Generated index.html");
        Ok(())
    }

    fn build_all(&self) -> Result<()> {
        std::fs::create_dir_all(&self.site_root)?;
        std::fs::create_dir_all(&self.pkg_dir)?;

        self.build_server()?;
        self.build_frontend()?;
        self.process_tailwind()?;
        self.copy_assets()?;
        self.generate_index_html()?;

        println!(" Build complete");
        Ok(())
    }

    fn metadata(&self) -> &MontrsMetadata {
        &self.meta
    }

    fn project_root(&self) -> &Path {
        &self.project_root
    }

    fn site_root(&self) -> &Path {
        &self.site_root
    }

    fn pkg_dir(&self) -> &Path {
        &self.pkg_dir
    }
}

/// Args for `cargo build` of the WASM frontend (hydrate client).
///
/// Order matters: the `--features` value must immediately follow the
/// `--features` flag; other flags come after.
fn frontend_build_args(
    pkg: &str,
    lib_features: &[String],
    lib_default_features: bool,
) -> Vec<String> {
    let mut args = vec![
        "build".to_string(),
        "--target".to_string(),
        "wasm32-unknown-unknown".to_string(),
        "--package".to_string(),
        pkg.to_string(),
        "--features".to_string(),
    ];
    let features = if lib_features.is_empty() {
        "hydrate".to_string()
    } else {
        lib_features.join(",")
    };
    // `--features` value must immediately follow the `--features` flag.
    args.push(features);
    if !lib_default_features {
        args.push("--no-default-features".to_string());
    }
    // A debug (unoptimized) WASM client is unusably large and slow in the
    // browser, so the frontend is always built with optimization.
    args.push("--release".to_string());
    args
}

/// Args for `cargo build` of the SSR server binary.
///
/// Order matters: the `--features` value must immediately follow the
/// `--features` flag; other flags come after.
fn server_build_args(
    pkg: &str,
    bin_features: &[String],
    bin_default_features: bool,
    release: bool,
) -> Vec<String> {
    let mut args = vec![
        "build".to_string(),
        "--package".to_string(),
        pkg.to_string(),
        "--features".to_string(),
    ];
    let features = if bin_features.is_empty() {
        "ssr".to_string()
    } else {
        bin_features.join(",")
    };
    // `--features` value must immediately follow the `--features` flag.
    args.push(features);
    if !bin_default_features {
        args.push("--no-default-features".to_string());
    }
    if release {
        args.push("--release".to_string());
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontend_args_keep_features_flag_with_value() {
        let args = frontend_build_args("website", &[], true);
        let joined = args.join(" ");
        assert!(
            joined.contains("--features hydrate --release"),
            "unexpected frontend args: {joined}"
        );
        // The `--features` flag must be immediately followed by its value.
        let idx = args
            .iter()
            .position(|a| a == "--features")
            .expect("--features flag present");
        assert_eq!(args[idx + 1], "hydrate", "args: {joined}");
    }

    #[test]
    fn frontend_args_respect_configured_features() {
        let args = frontend_build_args(
            "website",
            &["hydrate".to_string(), "foo".to_string()],
            false,
        );
        let joined = args.join(" ");
        assert!(
            joined.contains(
                "--features hydrate,foo --no-default-features --release"
            ),
            "unexpected frontend args: {joined}"
        );
    }

    #[test]
    fn server_args_keep_features_flag_with_value() {
        let args = server_build_args("website", &[], true, false);
        let joined = args.join(" ");
        assert_eq!(
            joined, "build --package website --features ssr",
            "unexpected server args"
        );
        let idx = args
            .iter()
            .position(|a| a == "--features")
            .expect("--features flag present");
        assert_eq!(args[idx + 1], "ssr", "args: {joined}");
    }

    #[test]
    fn server_args_append_release_after_features() {
        let args = server_build_args("website", &[], true, true);
        let joined = args.join(" ");
        assert_eq!(
            joined, "build --package website --features ssr --release",
            "unexpected server args"
        );
        let idx = args
            .iter()
            .position(|a| a == "--features")
            .expect("--features flag present");
        assert_eq!(args[idx + 1], "ssr", "args: {joined}");
    }
}
