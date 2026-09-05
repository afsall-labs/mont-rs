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

//! montrs-core/src/serve.rs: SSR server entry point.
//!
//! Provides `montrs_serve` — a single function call that replaces the ~40 lines
//! of boilerplate in every app's `main.rs`. Creates its own single-threaded
//! tokio runtime with LocalSet for Leptos SSR compatibility.

#[cfg(feature = "ssr")]
use crate::{AppConfig, Router};

/// Start an Axum SSR server backed by a MontRS Router.
///
/// Creates a single-threaded tokio runtime with a `LocalSet` to support
/// Leptos `spawn_local` during SSR rendering. Reads `MONTRS_SITE_ADDR`,
/// `MONTRS_SITE_ROOT`, and `MONTRS_SITE_PKG_DIR` from the environment.
///
/// # Example
/// ```rust,ignore
/// #[cfg(feature = "ssr")]
/// fn main() {
///     tracing_subscriber::fmt().with_env_filter("info").init();
///     let spec = app::build_spec();
///     montrs_core::serve::montrs_serve(spec.router, || view! { <app::App /> })
///         .unwrap();
/// }
/// ```
#[cfg(feature = "ssr")]
pub fn montrs_serve<C, F, IV>(
    router: Router<C>,
    app_fn: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    C: AppConfig + 'static,
    F: Fn() -> IV + Clone + Send + Sync + 'static,
    IV: leptos::prelude::IntoView + 'static,
{
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move { serve_inner(router, app_fn).await })
}

#[cfg(feature = "ssr")]
async fn serve_inner<C, F, IV>(
    router: Router<C>,
    app_fn: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    C: AppConfig + 'static,
    F: Fn() -> IV + Clone + Send + Sync + 'static,
    IV: leptos::prelude::IntoView + 'static,
{
    use axum::Router as AxumRouter;
    use leptos::prelude::*;
    use leptos_axum::LeptosRoutes;
    use tokio::task::LocalSet;
    use tower_http::services::ServeDir;

    // MontRS is the single source of truth for site config. Derive Leptos
    // runtime env vars from MONTRS_* values (set by the CLI from montrs.toml)
    // so the runtime no longer depends on `[package.metadata.leptos]`.
    let addr = std::env::var("MONTRS_SITE_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let site_root = std::env::var("MONTRS_SITE_ROOT")
        .unwrap_or_else(|_| "target/site".to_string());
    let pkg_dir = std::env::var("MONTRS_SITE_PKG_DIR")
        .unwrap_or_else(|_| "pkg".to_string());
    let output_name = std::env::var("MONTRS_OUTPUT_NAME")
        .unwrap_or_else(|_| "website".to_string());
    let reload_port = std::env::var("MONTRS_RELOAD_PORT")
        .unwrap_or_else(|_| "3001".to_string());

    unsafe {
        std::env::set_var("LEPTOS_OUTPUT_NAME", &output_name);
        std::env::set_var("LEPTOS_SITE_ADDR", &addr);
        std::env::set_var("LEPTOS_SITE_ROOT", &site_root);
        std::env::set_var("LEPTOS_SITE_PKG_DIR", &pkg_dir);
        std::env::set_var("LEPTOS_RELOAD_PORT", &reload_port);
    }

    let mut conf = get_configuration(None).unwrap();

    // The pkg dir must stay a site-root-relative URL path (e.g. "pkg").
    // An absolute filesystem path here (which the CLI could pass through)
    // leaks `\\?\C:\...` into the hydration bootstrap's `import()` specifier
    // and breaks WASM loading entirely.
    let relative_pkg = std::path::Path::new(&*conf.leptos_options.site_pkg_dir)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| pkg_dir.clone());
    conf.leptos_options.site_pkg_dir = relative_pkg.into();

    let axum_routes = router.to_axum_route_listings();

    let app = AxumRouter::new()
        .leptos_routes_with_context(
            &conf.leptos_options,
            axum_routes,
            {
                let r = router.clone();
                let leptos_options = conf.leptos_options.clone();
                move || {
                    provide_context(r.clone());
                    // The SSR shell reads `LeptosOptions` (output_name,
                    // site_root, pkg dir) to render the hydration bootstrap
                    // scripts that match the WASM bundle names.
                    provide_context(leptos_options.clone());
                }
            },
            app_fn,
        )
        .fallback_service(ServeDir::new(&site_root))
        .with_state(conf.leptos_options);

    let (host, port_str) = addr.rsplit_once(':').unwrap_or((&addr, "3000"));
    let mut port: u16 = port_str.parse().unwrap_or(3000);
    for _ in 0..100 {
        let bind_addr = format!("{host}:{port}");
        if let Ok(listener) = tokio::net::TcpListener::bind(&bind_addr).await {
            tracing::info!("listening on http://{host}:{port}");
            let local = LocalSet::new();
            let _guard = local.enter();
            axum::serve(listener, app.clone().into_make_service()).await?;
            return Ok(());
        }
        port += 1;
    }
    Err("Could not bind to any port in range".into())
}
