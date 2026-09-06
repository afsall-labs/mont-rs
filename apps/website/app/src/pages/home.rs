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

use crate::{copy::CopyButton, highlight::highlight_rust};
use leptos::prelude::*;
use montrs_icons::*;
use montrs_ui::{
    components::{input::Input, switch::Switch},
    prelude::*,
};

const HELLO_SNIPPET: &str = r#"use montrs_core::*;

struct HelloView;
impl RouteView for HelloView {
    fn render(&self) -> impl IntoView {
        view! { <h1>"Hello, MontRS!"</h1> }
    }
}

view_route! { HelloRoute, "/hello", HelloView }"#;

const VALIDATOR_SNIPPET: &str = r#"#[derive(Validator)]
pub struct Signup {
    #[validator(min_len = 3)]
    pub username: String,
}"#;

const APPSPEC_SNIPPET: &str = r#"{
  "app": "my-app",
  "target": "web",
  "routes": [
    { "path": "/", "view": "HomeRoute" },
    { "path": "/hello", "view": "HelloRoute" }
  ],
  "plates": ["auth", "tui"]
}"#;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <Hero />
        <BentoGrid />
        <GoldenPath />
        <Philosophy />
        <AgentFirst />
        <SectionLinks />
        <FinalCta />
    }
}

// ---------------------------------------------------------------------------
// Hero
// ---------------------------------------------------------------------------

#[component]
fn Hero() -> impl IntoView {
    let install_tab = RwSignal::new("cargo".to_string());

    view! {
        <section class="dot-grid glow-orange relative overflow-hidden">
            <div class="page-container pb-20 pt-16 sm:pt-24">
                <div class="mx-auto max-w-3xl text-center">
                    <div class="flex justify-center">
                        <span class="pill">
                            <span class="pill-accent">"Agent-first. Deterministic."</span>
                            "One AppSpec, three targets →"
                        </span>
                    </div>

                    <h1 class="mt-6 text-4xl font-bold tracking-tight sm:text-6xl">
                        "The deterministic full-stack framework for "
                        <span class="text-gradient">"Rust."</span>
                    </h1>

                    <p class="mx-auto mt-6 max-w-2xl text-lg leading-8 text-muted-foreground">
                        "MontRS gives you a unified, trait-driven environment for web,
                        desktop, and mobile — powered by Leptos, defined by a serializable
                        AppSpec, and natively understandable by AI agents. No magic.
                        No global state. Same input, same output, everywhere."
                    </p>

                    <div class="mt-10 flex flex-wrap items-center justify-center gap-4">
                        <a
                            href="/ui/components"
                            class="inline-flex items-center rounded-md bg-primary px-6 py-3 text-sm font-semibold text-primary-foreground shadow-sm transition-colors hover:bg-primary/90"
                        >
                            "Get Started"
                            <Icon glyph=Glyph::ArrowRight class="ml-2 h-4 w-4" />
                        </a>
                        <a
                            href="/docs"
                            class="inline-flex items-center rounded-md border border-border px-6 py-3 text-sm font-semibold transition-colors hover:bg-accent"
                        >
                            "Read the Docs"
                        </a>
                    </div>

                    <div class="mx-auto mt-12 max-w-xl">
                        <div class="terminal">
                            <div class="flex items-center justify-between">
                                <span class="font-mono text-[11px] uppercase tracking-wide text-muted-foreground">
                                    "Install MontRS"
                                </span>
                                <span class="kbd-hint">"cargo · curl"</span>
                            </div>
                            <div class="mt-3 flex items-center justify-between gap-4">
                                <span>
                                    <span class="terminal-prompt">"$"</span>
                                    {move || if install_tab.get() == "curl" {
                                        " curl -LsSf https://montrs.com/install.sh | sh"
                                    } else {
                                        " cargo install montrs-cli"
                                    }}
                                </span>
                                <Show
                                    when=move || install_tab.get() == "curl"
                                    fallback=|| view! {
                                        <CopyButton text="cargo install montrs-cli" label="Copy" />
                                    }
                                >
                                    <CopyButton text="curl -LsSf https://montrs.com/install.sh | sh" label="Copy" />
                                </Show>
                            </div>
                            <div class="mt-3 flex items-center gap-2">
                                <button
                                    type="button"
                                    class=move || {
                                        let base = "rounded-full border px-3 py-1 text-xs font-medium transition-colors";
                                        if install_tab.get() == "cargo" {
                                            format!("{base} border-primary bg-primary/10 text-primary")
                                        } else {
                                            format!("{base} border-border text-muted-foreground hover:bg-accent")
                                        }
                                    }
                                    on:click=move |_| install_tab.set("cargo".to_string())
                                >
                                    "cargo"
                                </button>
                                <button
                                    type="button"
                                    class=move || {
                                        let base = "rounded-full border px-3 py-1 text-xs font-medium transition-colors";
                                        if install_tab.get() == "curl" {
                                            format!("{base} border-primary bg-primary/10 text-primary")
                                        } else {
                                            format!("{base} border-border text-muted-foreground hover:bg-accent")
                                        }
                                    }
                                    on:click=move |_| install_tab.set("curl".to_string())
                                >
                                    "curl"
                                </button>
                                <span class="ml-auto text-xs text-muted-foreground">
                                    "then: "
                                    <code class="terminal-prompt">"montrs new my-app"</code>
                                </span>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="mx-auto mt-14 max-w-3xl">
                    <CodeWindow
                        tab="hello.rs"
                        body=move || highlight_rust(HELLO_SNIPPET)
                    />
                </div>

                <div class="mt-12 flex flex-wrap items-center justify-center gap-x-8 gap-y-3 text-sm text-muted-foreground">
                    <span class="inline-flex items-center gap-2">
                        <Icon glyph=Glyph::Monitor class="h-4 w-4" />
                        "Web"
                    </span>
                    <span class="inline-flex items-center gap-2">
                        <Icon glyph=Glyph::MonitorSmartphone class="h-4 w-4" />
                        "Desktop"
                    </span>
                    <span class="inline-flex items-center gap-2">
                        <Icon glyph=Glyph::Smartphone class="h-4 w-4" />
                        "Mobile"
                    </span>
                    <span class="inline-flex items-center gap-2">
                        <Icon glyph=Glyph::Server class="h-4 w-4" />
                        "Server"
                    </span>
                    <span class="hidden h-4 w-px bg-border sm:block"></span>
                    <span class="hidden sm:inline-flex sm:items-center sm:gap-2">
                        <span class="kbd-hint">"WASM"</span>
                        <span class="kbd-hint">"wgpu"</span>
                        <span class="kbd-hint">"axum"</span>
                        <span class="kbd-hint">"Leptos"</span>
                    </span>
                </div>
            </div>
        </section>
    }
}

#[component]
fn CodeWindow(
    tab: &'static str,
    #[prop(into)] body: TextProp,
) -> impl IntoView {
    view! {
        <div class="code-window">
            <div class="code-window-bar">
                <span class="traffic-light traffic-light-red"></span>
                <span class="traffic-light traffic-light-yellow"></span>
                <span class="traffic-light traffic-light-green"></span>
                <span class="code-window-tab">{tab}</span>
            </div>
            <pre class="code-window-body text-left" inner_html=move || body.get()></pre>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Interactive bento grid
// ---------------------------------------------------------------------------

#[component]
fn BentoGrid() -> impl IntoView {
    let switch_on = RwSignal::new(true);
    let input_value = RwSignal::new(String::new());
    let ball = RwSignal::new(0.0);
    let ball_playing = RwSignal::new(false);

    let play_ball = move |_| {
        if ball_playing.get() {
            return;
        }
        ball_playing.set(true);
        let start = montrs_motion::FrameLoop::now();
        montrs_motion::FrameLoop::on_frame(move || {
            let elapsed = montrs_motion::FrameLoop::now() - start;
            let t = (elapsed / 1.2).min(1.0);
            let eased = 1.0 - (1.0 - t) * (1.0 - t);
            ball.set(eased * 64.0);
            if elapsed > 1.2 {
                ball_playing.set(false);
                false
            } else {
                true
            }
        });
    };

    let ball_style = move || {
        format!("transform: translateY({}px); transition: none;", ball.get())
    };

    view! {
        <section class="border-t border-border py-20">
            <div class="page-container">
                <div class="mx-auto max-w-2xl text-center">
                    <h2 class="text-3xl font-bold tracking-tight sm:text-4xl">
                        "The framework, demonstrated"
                    </h2>
                    <p class="mt-4 text-muted-foreground">
                        "Every layer of MontRS, live. Click around — it's all reactive."
                    </p>
                </div>

                <div class="mt-12 grid grid-cols-1 gap-4 md:grid-cols-3">
                    // UI kit
                    <div class="showcase-card reveal flex flex-col p-6 md:col-span-2">
                        <div class="flex items-center justify-between">
                            <div>
                                <h3 class="font-semibold">"UI kit"</h3>
                                <p class="mt-1 text-sm text-muted-foreground">
                                    "91 shadcn-inspired components, reactive out of the box."
                                </p>
                            </div>
                            <a
                                href="/ui/components"
                                class="rounded-md border border-border px-3 py-1.5 text-xs font-medium transition-colors hover:bg-accent"
                            >"Browse"</a>
                        </div>
                        <div class="mt-6 flex flex-wrap items-center gap-4">
                            <button
                                type="button"
                                class="inline-flex h-9 items-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-sm transition-colors hover:bg-primary/90"
                            >
                                "Primary"
                            </button>
                            <button
                                type="button"
                                class="inline-flex h-9 items-center rounded-md border border-border px-4 py-2 text-sm font-medium transition-colors hover:bg-accent"
                            >
                                "Secondary"
                            </button>
                            <Switch checked=switch_on />
                            <span class="text-xs text-muted-foreground">
                                {move || if switch_on.get() { "On" } else { "Off" }}
                            </span>
                        </div>
                        <div class="mt-4 flex max-w-xs items-center gap-2">
                            <Input
                                placeholder="Type anything…"
                                value=input_value
                                class="flex-1"
                            />
                        </div>
                    </div>

                    // Icons
                    <div class="showcase-card reveal flex flex-col p-6">
                        <div class="flex items-center justify-between">
                            <div>
                                <h3 class="font-semibold">"Icons"</h3>
                                <p class="mt-1 text-sm text-muted-foreground">
                                    "1,600+ Lucide icons as Leptos components."
                                </p>
                            </div>
                            <a
                                href="/ui/icons"
                                class="rounded-md border border-border px-3 py-1.5 text-xs font-medium transition-colors hover:bg-accent"
                            >"Browse"</a>
                        </div>
                        <div class="mt-6 grid grid-cols-6 gap-2">
                            {[
                                Glyph::Heart,
                                Glyph::Rocket,
                                Glyph::Zap,
                                Glyph::Shield,
                                Glyph::Palette,
                                Glyph::Bot,
                                Glyph::Puzzle,
                                Glyph::Layers,
                                Glyph::Cpu,
                                Glyph::Globe,
                                Glyph::Key,
                                Glyph::WandSparkles,
                            ].into_iter().map(|g| view! {
                                <div class="flex aspect-square items-center justify-center rounded-md border border-border text-muted-foreground">
                                    <Icon glyph=g class="h-4 w-4" />
                                </div>
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>

                    // Motion
                    <div class="showcase-card reveal flex flex-col p-6">
                        <div class="flex items-center justify-between">
                            <div>
                                <h3 class="font-semibold">"Motion"</h3>
                                <p class="mt-1 text-sm text-muted-foreground">
                                    "Springs, keyframes, gestures — one API."
                                </p>
                            </div>
                            <a
                                href="/ui/motion"
                                class="rounded-md border border-border px-3 py-1.5 text-xs font-medium transition-colors hover:bg-accent"
                            >"Demo"</a>
                        </div>
                        <div class="mt-6 flex flex-1 items-end justify-center rounded-md border border-border bg-background p-6">
                            <div
                                class="h-8 w-8 rounded-full bg-primary"
                                style=ball_style
                            ></div>
                        </div>
                        <button
                            type="button"
                            class="mt-4 inline-flex h-9 items-center justify-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-sm transition-colors hover:bg-primary/90"
                            on:click=play_ball
                            disabled=move || ball_playing.get()
                        >
                            <Icon glyph=Glyph::Play class="h-4 w-4" />
                            "Play spring"
                        </button>
                    </div>

                    // Validator
                    <div class="showcase-card reveal flex flex-col p-6">
                        <h3 class="font-semibold">"Compile-time validation"</h3>
                        <p class="mt-1 text-sm text-muted-foreground">
                            "Models checked before your app ever runs."
                        </p>
                        <div class="code-window mt-5">
                            <pre class="code-window-body text-left" inner_html=move || highlight_rust(VALIDATOR_SNIPPET)></pre>
                        </div>
                    </div>

                    // AppSpec
                    <div class="showcase-card reveal flex flex-col p-6">
                        <h3 class="font-semibold">"Your app, as data"</h3>
                        <p class="mt-1 text-sm text-muted-foreground">
                            "One serializable AppSpec. Portable, inspectable, testable."
                        </p>
                        <div class="code-window mt-5">
                            <pre class="code-window-body text-left" inner_html=move || crate::highlight::escape_html(APPSPEC_SNIPPET)></pre>
                        </div>
                    </div>

                    // TestRuntime
                    <div class="showcase-card reveal flex flex-col p-6">
                        <h3 class="font-semibold">"Deterministic testing"</h3>
                        <p class="mt-1 text-sm text-muted-foreground">
                            "Your whole app spec runs in-process. No browser, no network."
                        </p>
                        <div class="terminal mt-5 flex-1">
                            <p class="text-sm">
                                <span class="terminal-prompt">"$"</span>
                                " montrs test"
                            </p>
                            <p class="mt-2 text-sm text-green-600 dark:text-green-400">
                                "  ✓ routes resolved (7)"
                            </p>
                            <p class="text-sm text-green-600 dark:text-green-400">
                                "  ✓ plates registered (12)"
                            </p>
                            <p class="text-sm text-green-600 dark:text-green-400">
                                "  ✓ all deterministic — 48 passed"
                            </p>
                        </div>
                    </div>

                    // Auth + ORM
                    <div class="showcase-card reveal flex flex-col p-6">
                        <h3 class="font-semibold">"Auth · ORM · services"</h3>
                        <p class="mt-1 text-sm text-muted-foreground">
                            "Plugin-based auth and a SQL-first ORM, gated behind traits."
                        </p>
                        <div class="mt-5 grid grid-cols-2 gap-2 text-xs">
                            <a href="/auth" class="rounded-md border border-border p-3 transition-colors hover:bg-accent">
                                <Icon glyph=Glyph::KeyRound class="mb-2 h-4 w-4 text-primary" />
                                <p class="font-medium">"Auth"</p>
                                <p class="mt-1 text-muted-foreground">"OAuth · 2FA · API keys · SSO"</p>
                            </a>
                            <a href="/orm" class="rounded-md border border-border p-3 transition-colors hover:bg-accent">
                                <Icon glyph=Glyph::Database class="mb-2 h-4 w-4 text-primary" />
                                <p class="font-medium">"ORM"</p>
                                <p class="mt-1 text-muted-foreground">"SQL-first, backend-agnostic"</p>
                            </a>
                        </div>
                    </div>

                    // TUI
                    <div class="showcase-card reveal flex flex-col p-6">
                        <h3 class="font-semibold">"Terminal UI"</h3>
                        <p class="mt-1 text-sm text-muted-foreground">
                            "21 renderables, one renderer."
                        </p>
                        <div class="terminal mt-5 flex-1">
                            <p class="text-sm">
                                <span class="terminal-prompt">"❯ "</span>
                                " montrs serve"
                            </p>
                            <p class="mt-1 text-sm text-muted-foreground">
                                "  dev server on 127.0.0.1:3000"
                            </p>
                            <p class="text-sm text-muted-foreground">
                                "  watching 48 packages…"
                            </p>
                            <p class="text-sm text-green-600 dark:text-green-400">
                                "  ✓ rebuilt in 42ms"
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ---------------------------------------------------------------------------
// Golden Path
// ---------------------------------------------------------------------------

#[component]
fn GoldenPath() -> impl IntoView {
    let steps = [
        (
            "1",
            "Scaffold",
            "Start with a pre-configured workspace.",
            "montrs new my-app",
            Glyph::FolderPlus,
        ),
        (
            "2",
            "Define",
            "Models validated at compile time.",
            "#[derive(Validator)]",
            Glyph::FileCheck,
        ),
        (
            "3",
            "Implement",
            "Features are Plates; Routes bundle Loader, Action, View.",
            "impl Plate for TodoPlate",
            Glyph::Blocks,
        ),
        (
            "4",
            "Verify",
            "TestRuntime runs your entire app spec deterministically.",
            "montrs test",
            Glyph::FlaskConical,
        ),
        (
            "5",
            "Ship",
            "One command, every target.",
            "montrs build",
            Glyph::Rocket,
        ),
    ];

    view! {
        <section class="border-t border-border py-20">
            <div class="page-container">
                <div class="mx-auto max-w-2xl text-center">
                    <h2 class="text-3xl font-bold tracking-tight sm:text-4xl">
                        "Five steps. Zero guesswork."
                    </h2>
                    <p class="mt-4 text-muted-foreground">
                        "The Golden Path is the recommended workflow — from empty folder to shipped app."
                    </p>
                </div>

                <div class="mt-12 grid grid-cols-1 gap-4 md:grid-cols-5">
                    {steps.into_iter().map(|(num, title, desc, cmd, icon)| view! {
                        <div class="showcase-card reveal p-5">
                            <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-primary">
                                <Icon glyph=icon class="h-5 w-5" />
                            </div>
                            <div class="mt-4 flex items-center gap-2">
                                <span class="font-mono text-xs text-muted-foreground">{num}</span>
                                <h3 class="font-semibold">{title}</h3>
                            </div>
                            <p class="mt-2 text-sm text-muted-foreground">{desc}</p>
                            <div class="terminal mt-4 px-3 py-2 text-xs">
                                <span class="terminal-prompt">"$"</span>
                                " "{cmd}
                            </div>
                        </div>
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </section>
    }
}

// ---------------------------------------------------------------------------
// Philosophy
// ---------------------------------------------------------------------------

#[component]
fn Philosophy() -> impl IntoView {
    let items = [
        (
            Glyph::Timer,
            "Deterministic",
            "Same input, same output. In production, in tests, on every \
             platform.",
        ),
        (
            Glyph::Puzzle,
            "Trait-driven boundaries",
            "Features are Plates with explicit interfaces. Change behavior by \
             implementing a trait — ORM, auth, rendering.",
        ),
        (
            Glyph::Bot,
            "Agent-first",
            "Structured snapshots, agent.json, and skills make MontRS apps \
             natively readable by AI agents.",
        ),
        (
            Glyph::WandSparkles,
            "No magic",
            "Explicit registration over reflection. No global state, ever.",
        ),
        (
            Glyph::Braces,
            "One AppSpec",
            "Your entire app is a serializable spec: portable, inspectable, \
             testable.",
        ),
        (
            Glyph::Cpu,
            "Rust end to end",
            "Leptos reactivity, the type system, and compile-time validation \
             catch bugs before runtime.",
        ),
    ];

    view! {
        <section class="border-t border-border py-20">
            <div class="page-container">
                <div class="mx-auto max-w-2xl text-center">
                    <h2 class="text-3xl font-bold tracking-tight sm:text-4xl">
                        "Predictable architecture, by design."
                    </h2>
                    <p class="mt-4 text-muted-foreground">
                        "Building complex apps requires more than a UI library."
                    </p>
                </div>

                <div class="mt-12 grid grid-cols-1 gap-4 md:grid-cols-3">
                    {items.into_iter().map(|(icon, title, desc)| view! {
                        <div class="showcase-card reveal p-6">
                            <Icon glyph=icon class="h-6 w-6 text-primary" />
                            <h3 class="mt-4 font-semibold">{title}</h3>
                            <p class="mt-2 text-sm leading-6 text-muted-foreground">{desc}</p>
                        </div>
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </section>
    }
}

// ---------------------------------------------------------------------------
// Agent-first
// ---------------------------------------------------------------------------

#[component]
fn AgentFirst() -> impl IntoView {
    view! {
        <section class="border-t border-border py-20">
            <div class="page-container">
                <div class="grid grid-cols-1 gap-12 lg:grid-cols-2 lg:items-center">
                    <div>
                        <h2 class="text-3xl font-bold tracking-tight sm:text-4xl">
                            "Built for humans. Native to agents."
                        </h2>
                        <p class="mt-4 text-lg leading-8 text-muted-foreground">
                            "Every MontRS project exposes a machine-readable spec:
                            plates, routes, tools, and error tracking with suggested fixes.
                            Metadata markers and composable skills make your codebase
                            readable by AI coding partners without a single prompt."
                        </p>
                        <div class="mt-6 space-y-3 text-sm">
                            <div class="flex items-center gap-3 rounded-md border border-border px-4 py-3">
                                <Icon glyph=Glyph::Braces class="h-4 w-4 shrink-0 text-primary" />
                                <span class="font-mono">"agent.json"</span>
                                <span class="text-muted-foreground">"— plates, routes, invariants"</span>
                            </div>
                            <div class="flex items-center gap-3 rounded-md border border-border px-4 py-3">
                                <Icon glyph=Glyph::Wrench class="h-4 w-4 shrink-0 text-primary" />
                                <span class="font-mono">"@agent-tool"</span>
                                <span class="text-muted-foreground">"— agent-callable functions"</span>
                            </div>
                            <div class="flex items-center gap-3 rounded-md border border-border px-4 py-3">
                                <Icon glyph=Glyph::Blocks class="h-4 w-4 shrink-0 text-primary" />
                                <span class="font-mono">"@agent-skill"</span>
                                <span class="text-muted-foreground">"— composable workflows"</span>
                            </div>
                        </div>
                        <a
                            href="/ai"
                            class="mt-6 inline-flex items-center text-sm font-medium text-primary hover:underline"
                        >
                            "Explore the AI Kit →"
                        </a>
                    </div>

                    <div class="code-window">
                        <div class="code-window-bar">
                            <span class="traffic-light traffic-light-red"></span>
                            <span class="traffic-light traffic-light-yellow"></span>
                            <span class="traffic-light traffic-light-green"></span>
                            <span class="code-window-tab">"agent terminal"</span>
                        </div>
                        <pre class="code-window-body text-left">
                            <span class="terminal-prompt">"$"</span>
                            " montrs agent doctor"{"\n"}
                            <span class="token-string">"  ✅ root Cargo.toml found"</span>{"\n"}
                            <span class="token-string">"  ✅ .agent directory exists"</span>{"\n"}
                            <span class="token-string">"  ✅ all tracked errors resolved"</span>{"\n"}
                            <span class="token-string">"  ✅ rust toolchain available"</span>{"\n\n"}
                            <span class="terminal-prompt">"$"</span>
                            " montrs agent check"{"\n"}
                            <span class="token-string">"  ✓ 48 packages · 12 plates · 7 routes"</span>{"\n"}
                            <span class="token-string">"  ✓ invariants satisfied"</span>{"\n"}
                            <span class="token-comment">"  # skills: fixing-errors, adding-features"</span>
                        </pre>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ---------------------------------------------------------------------------
// Section links
// ---------------------------------------------------------------------------

#[component]
fn SectionLinks() -> impl IntoView {
    let sections = [
        (
            "/docs",
            Glyph::BookOpen,
            "Docs",
            "40+ packages, templates, and the CLI",
        ),
        (
            "/auth",
            Glyph::KeyRound,
            "Auth",
            "OAuth, 2FA, passkeys, sessions — better-auth style",
        ),
        (
            "/runtime",
            Glyph::Zap,
            "Runtime",
            "Deno-inspired ops and memory-optimized execution",
        ),
        (
            "/ai",
            Glyph::Bot,
            "AI Kit",
            "Agentic framework, spec snapshots, skills",
        ),
        (
            "/orm",
            Glyph::Database,
            "ORM",
            "SQL-first, backend-agnostic data layer",
        ),
    ];

    view! {
        <section class="border-t border-border py-20">
            <div class="page-container">
                <div class="mx-auto max-w-2xl text-center">
                    <h2 class="text-3xl font-bold tracking-tight sm:text-4xl">
                        "Explore the framework."
                    </h2>
                    <p class="mt-4 text-muted-foreground">
                        "One framework. Five pillars."
                    </p>
                </div>
                <div class="mt-12 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-5">
                    {sections.into_iter().map(|(href, icon, title, desc)| view! {
                        <a href=href class="showcase-card reveal flex flex-col items-center p-6 text-center">
                            <Icon glyph=icon class="h-6 w-6 text-primary" />
                            <h3 class="mt-3 font-semibold">{title}</h3>
                            <p class="mt-1 text-sm text-muted-foreground">{desc}</p>
                        </a>
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </section>
    }
}

// ---------------------------------------------------------------------------
// Final CTA
// ---------------------------------------------------------------------------

#[component]
fn FinalCta() -> impl IntoView {
    view! {
        <section class="border-t border-border py-24">
            <div class="page-container text-center">
                <div class="glow-orange mx-auto max-w-2xl rounded-2xl border border-border p-10">
                    <h2 class="text-3xl font-bold tracking-tight sm:text-4xl">
                        "Stop debugging nondeterminism."
                        <span class="block text-primary">"Start shipping."</span>
                    </h2>
                    <p class="mx-auto mt-4 max-w-xl text-muted-foreground">
                        "Describe it once. Run it everywhere — web, desktop, and mobile,
                        with the same AppSpec, the same tests, the same output."
                    </p>
                    <div class="mt-8 flex flex-wrap items-center justify-center gap-4">
                        <a
                            href="/ui/components"
                            class="inline-flex items-center rounded-md bg-primary px-6 py-3 text-sm font-semibold text-primary-foreground shadow-sm transition-colors hover:bg-primary/90"
                        >
                            "Get Started"
                        </a>
                        <a
                            href="https://github.com/montrs/montrs"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="inline-flex items-center rounded-md border border-border px-6 py-3 text-sm font-semibold transition-colors hover:bg-accent"
                        >
                            <Icon glyph=Glyph::Star class="mr-2 h-4 w-4" />
                            "Star on GitHub"
                        </a>
                    </div>
                </div>
            </div>
        </section>
    }
}
