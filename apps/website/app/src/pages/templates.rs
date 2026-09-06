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

const TEMPLATES: &[(&str, &str, &str, &str)] = &[
    ("default", "Single-app web + e2e", "montrs new my-app", "The recommended starting point: a reactive SSR web app with tests and dark mode out of the box."),
    ("saas", "Full SaaS layout", "montrs new my-app --template saas", "Auth-ready structure with users, orgs, and billing scaffolding."),
    ("todo", "The classic TodoPlate", "montrs new my-app --template todo", "A working CRUD app demonstrating Plates, Routes, and the ORM."),
    ("api", "Headless API service", "montrs new my-api --template api", "A minimal axum server with health checks and structured logging."),
    ("desktop", "Desktop shell", "montrs new my-app --template desktop", "A winit/wgpu desktop window driven by the same AppSpec."),
    ("monorepo", "Workspace with multiple apps", "montrs new my-app --template monorepo", "Shared packages and multiple apps in one workspace."),
];

#[component]
pub fn Templates() -> impl IntoView {
    view! {
        <div class="page-container py-12">
            <div class="mb-10">
                <h1 class="text-3xl font-bold tracking-tight">"Templates"</h1>
                <p class="mt-2 max-w-2xl text-muted-foreground">
                    "Start from a pre-configured workspace with one command.
                    Every template includes Tailwind, dark mode, tests, and the
                    montrs task runner wired up in montrs.toml."
                </p>
            </div>

            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                {TEMPLATES.iter().map(|(name, tag, cmd, desc)| view! {
                    <div class="showcase-card reveal p-6">
                        <div class="flex items-center justify-between gap-2">
                            <span class="font-mono text-sm font-medium text-primary">{*name}</span>
                            <span class="rounded-full border border-border px-2 py-0.5 text-[10px] text-muted-foreground">{*tag}</span>
                        </div>
                        <p class="mt-2 text-sm leading-6 text-muted-foreground">{*desc}</p>
                        <div class="terminal mt-4 flex items-center justify-between gap-3 px-3 py-2 text-xs">
                            <span class="truncate">
                                <span class="terminal-prompt">"$"</span>
                                " "{*cmd}
                            </span>
                            <CopyButton text=cmd.to_string() label="Copy" />
                        </div>
                    </div>
                }).collect::<Vec<_>>()}
            </div>

            <div class="mt-12 grid grid-cols-1 gap-6 lg:grid-cols-2">
                <div class="code-window">
                    <div class="code-window-bar">
                        <span class="traffic-light traffic-light-red"></span>
                        <span class="traffic-light traffic-light-yellow"></span>
                        <span class="traffic-light traffic-light-green"></span>
                        <span class="code-window-tab">"montrs.toml"</span>
                    </div>
                    <pre class="code-window-body text-left">
                        <span class="token-comment">"# tasks run from montrs.toml — no Makefile needed"</span>{"\n"}
                        <span class="token-keyword">"[tasks]"</span>{"\n"}
                        "dev = \"montrs serve\""{"\n"}
                        "test = \"cargo test --workspace\""{"\n"}
                        <span class="token-keyword">"[tasks.ci]"</span>{"\n"}
                        "depends = [\"fmt\", \"lint\", \"test\"]"
                    </pre>
                </div>
                <div class="space-y-3 text-sm text-muted-foreground">
                    {[
                        (Glyph::Check, "Reactive SSR with Leptos, WASM hydration, and dark mode"),
                        (Glyph::Check, "Tailwind v4 pre-wired to the MontRS design tokens"),
                        (Glyph::Check, "Deterministic TestRuntime + Playwright E2E scaffold"),
                        (Glyph::Check, "Built-in task runner driven by montrs.toml"),
                        (Glyph::Check, "Favicons, assets pipeline, and hot reload included"),
                    ].into_iter().map(|(icon, text)| view! {
                        <div class="flex items-center gap-3 rounded-md border border-border px-4 py-3">
                            <Icon glyph=icon class="h-4 w-4 shrink-0 text-success" />
                            {text}
                        </div>
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}