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

use crate::copy::CopyButton;
use leptos::prelude::*;
use montrs_icons::*;
use montrs_ui::prelude::*;

const PACKAGES: &[(&str, &str, &str)] = &[
    (
        "core",
        "Foundational traits — Plate, Route, AppSpec, AgentError",
        "Core",
    ),
    ("platform", "Target enum, PlatformAdapter trait", "Core"),
    ("metadata", "montrs.toml single source of truth", "Core"),
    ("cli", "The `montrs` command", "Core"),
    ("montrs", "Facade crate — re-exports", "Core"),
    ("build", "Build pipeline facade", "Core"),
    ("build-core", "BuildPipeline trait + BuildConfig", "Core"),
    ("build-serve", "Dev server (axum static serving)", "Core"),
    ("build-watch", "File watcher with debounced rebuild", "Core"),
    ("runner", "Custom task runner config", "Core"),
    ("validator", "Compile-time validation (derive)", "Core"),
    ("ui", "91 shadcn-inspired components", "Experience"),
    (
        "icons",
        "1,600+ Lucide icons as Leptos components",
        "Experience",
    ),
    (
        "motion",
        "Springs, tweens, keyframes, gestures",
        "Experience",
    ),
    ("haptics", "Cross-platform haptic feedback", "Experience"),
    ("hotkeys-core", "Shortcut parsing/matching", "Experience"),
    ("hotkeys-web", "Browser/WASM hotkey adapter", "Experience"),
    (
        "i18n",
        "Internationalization, plurals, scoping",
        "Experience",
    ),
    (
        "table-core",
        "Headless table state + row models",
        "Experience",
    ),
    ("image-core", "Validated image request specs", "Experience"),
    (
        "image-optimizer",
        "Bounded image optimization policy",
        "Experience",
    ),
    (
        "orm",
        "SQL-first, backend-agnostic DB abstraction",
        "Full-stack",
    ),
    (
        "auth",
        "Email/password, OAuth, 2FA, sessions, RBAC",
        "Full-stack",
    ),
    (
        "services",
        "Service supervisor (daemon, retries, cron)",
        "Full-stack",
    ),
    ("proxy", "Reverse proxy routing", "Full-stack"),
    ("web", "Web platform adapter (WASM)", "Full-stack"),
    ("desktop", "Native desktop (wry webview)", "Full-stack"),
    ("mobile", "Mobile platform adapter", "Full-stack"),
    (
        "renderer",
        "Renderer trait + geometry (wgpu/tiny-skia)",
        "Full-stack",
    ),
    (
        "runtime",
        "Native Rust runtime (Deno-inspired ops)",
        "Runtime",
    ),
    ("log", "Structured log store with retention", "Runtime"),
    ("env", "Env parsing + .env loading + Tera", "Runtime"),
    (
        "state",
        "Deterministic stores, machines, history",
        "Runtime",
    ),
    ("command", "Typed command registry", "Runtime"),
    ("content", "Typed Markdown content collections", "Runtime"),
    ("test", "Deterministic TestRuntime + E2E", "Tooling"),
    ("bench", "Statistical benchmarking", "Tooling"),
    ("agent", "Agent spec, snapshots, error tracking", "Tooling"),
    ("agentignore", ".agentignore patterns", "Tooling"),
    ("tool", "Tool version manager (6 backends)", "Tooling"),
    ("lockfile", "Deterministic tool version locking", "Tooling"),
    ("registry", "Tool registry (baked + floating)", "Tooling"),
    ("plugin", "Tool plugin system (asdf/vfox)", "Tooling"),
    ("shell", "Shell integration + shims", "Tooling"),
    ("sigstore", "GitHub attestation, cosign, SLSA", "Tooling"),
    ("deps", "Dependency freshness checking", "Tooling"),
    ("fmt", "Custom formatter for Rust + view!", "Tooling"),
    ("prdoc", "PR doc parser/generator/changelog", "Tooling"),
];

const CLI_COMMANDS: &[(&str, &str)] = &[
    ("montrs new", "Scaffold a new app or template"),
    ("montrs serve", "Run the dev server with hot reload"),
    ("montrs build", "Build for production (WASM + SSR)"),
    ("montrs test", "Run all workspace tests"),
    ("montrs fmt", "Format all Rust + view! code"),
    ("montrs install", "Install toolchain prerequisites"),
    ("montrs agent doctor", "Full agent health check"),
    ("montrs agent check", "Agent-level diagnostics"),
    ("montrs mcp serve", "Serve MCP tools for agent tool calls"),
];

const TEMPLATES: &[(&str, &str)] = &[
    ("default", "A single-app workspace with web + e2e"),
    ("api", "Headless API service"),
    ("desktop", "Desktop shell (winit/wgpu)"),
    ("saas", "Full SaaS layout with auth-ready structure"),
    ("todo", "The classic TodoPlate example"),
    ("monorepo", "Workspace with multiple apps"),
];

#[component]
pub fn Docs() -> impl IntoView {
    let filter = RwSignal::new("All".to_string());
    let groups = [
        "All",
        "Core",
        "Experience",
        "Full-stack",
        "Runtime",
        "Tooling",
    ];

    let filtered = move || {
        let f = filter.get();
        PACKAGES
            .iter()
            .copied()
            .filter(|(_, _, group)| f == "All" || *group == f)
            .collect::<Vec<_>>()
    };

    view! {
        <div class="page-container py-12">
            <div class="mb-10">
                <h1 class="text-3xl font-bold tracking-tight">"Documentation"</h1>
                <p class="mt-2 max-w-2xl text-muted-foreground">
                    "One framework. Forty packages. Every layer documented,
                    trait-driven, and deterministic."
                </p>
            </div>

            <div class="mb-6 flex flex-wrap gap-2">
                {groups.into_iter().map(|g| {
                    let g2 = g.to_string();
                    let g3 = g2.clone();
                    let set_filter = filter;
                    view! {
                        <button
                            type="button"
                            class=move || {
                                let base = "rounded-full border px-4 py-1.5 text-sm font-medium transition-colors";
                                if set_filter.get() == g2 {
                                    format!("{base} border-primary bg-primary/10 text-primary")
                                } else {
                                    format!("{base} border-border text-muted-foreground hover:bg-accent hover:text-foreground")
                                }
                            }
                            on:click=move |_| set_filter.set(g3.clone())
                        >{g}</button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                <For
                    each=move || filtered()
                    key=|p| p.0
                    children=move |(name, purpose, group)| {
                        view! {
                            <div class="showcase-card p-4">
                                <div class="flex items-center justify-between gap-2">
                                    <span class="font-mono text-sm font-medium text-primary">{name}</span>
                                    <span class="rounded-full border border-border px-2 py-0.5 text-[10px] text-muted-foreground">{group}</span>
                                </div>
                                <p class="mt-2 text-sm text-muted-foreground">{purpose}</p>
                            </div>
                        }
                    }
                />
            </div>

            <div class="mt-16 grid grid-cols-1 gap-8 lg:grid-cols-2">
                <section>
                    <h2 class="text-2xl font-bold tracking-tight">"Templates"</h2>
                    <p class="mt-1 text-sm text-muted-foreground">
                        "Start from a pre-configured workspace with one command."
                    </p>
                    <div class="mt-5 space-y-3">
                        {TEMPLATES.iter().map(|(name, desc)| view! {
                            <div class="flex items-center justify-between gap-3 rounded-lg border border-border bg-card p-4">
                                <div class="flex items-center gap-3">
                                    <Icon glyph=Glyph::Folder class="h-4 w-4 shrink-0 text-primary" />
                                    <div>
                                        <p class="font-mono text-sm">{*name}</p>
                                        <p class="text-xs text-muted-foreground">{*desc}</p>
                                    </div>
                                </div>
                                <CopyButton text=format!("montrs new my-app --template {}", name) label="Copy" />
                            </div>
                        }).collect::<Vec<_>>()}
                    </div>
                </section>

                <section>
                    <h2 class="text-2xl font-bold tracking-tight">"CLI"</h2>
                    <p class="mt-1 text-sm text-muted-foreground">
                        "Everything you need, one binary."
                    </p>
                    <div class="mt-5 space-y-3">
                        {CLI_COMMANDS.iter().map(|(cmd, desc)| view! {
                            <div class="flex items-center justify-between gap-3 rounded-lg border border-border bg-card p-4">
                                <div class="flex items-center gap-3">
                                    <span class="terminal-prompt">"$"</span>
                                    <div>
                                        <p class="font-mono text-sm">{*cmd}</p>
                                        <p class="text-xs text-muted-foreground">{*desc}</p>
                                    </div>
                                </div>
                                <CopyButton text=cmd.to_string() label="Copy" />
                            </div>
                        }).collect::<Vec<_>>()}
                    </div>
                </section>
            </div>
        </div>
    }
}
