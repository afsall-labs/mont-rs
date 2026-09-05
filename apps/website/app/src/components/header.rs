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

use leptos::prelude::*;
use montrs_core::nav::*;
use montrs_icons::*;
use montrs_ui::prelude::*;

// NOTE: nav links use plain anchors with a `use_navigate` click handler.
// `use_navigate` (RouterContext::navigate) updates the internal location AND
// completes the browser-history navigation immediately — unlike `<A>`, whose
// global anchor interception defers the URL update until the leptos
// `<Routes>` tree resolves. Our custom RouterOutlet never resolves routes,
// so `<A>` would stop updating the address bar entirely.

#[component]
pub fn Header() -> impl IntoView {
    let theme = use_theme();
    let theme_open = RwSignal::new(false);
    let mobile_open = RwSignal::new(false);
    let navigate = use_navigate();

    let theme_icon = Memo::new(move |_| match theme.get() {
        ThemeMode::Light => Glyph::Sun,
        ThemeMode::Dark => Glyph::Moon,
        ThemeMode::System => Glyph::Monitor,
    });

    let nav_links = [
        ("/", "Home"),
        ("/ui", "UI"),
        ("/docs", "Docs"),
        ("/auth", "Auth"),
        ("/runtime", "Runtime"),
        ("/ai", "AI Kit"),
        ("/orm", "ORM"),
    ];

    let theme_modes = [
        ("System", ThemeMode::System, Glyph::Monitor),
        ("Light", ThemeMode::Light, Glyph::Sun),
        ("Dark", ThemeMode::Dark, Glyph::Moon),
    ];

    view! {
        <header class="sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
            <div class="page-container flex h-16 items-center justify-between">
                <div class="flex items-center gap-6">
                    <a
                        href="/"
                        class="flex items-center gap-2 text-lg font-bold"
                        on:click={
                            let nav = navigate.clone();
                            move |ev| {
                                ev.prevent_default();
                                nav("/", Default::default());
                            }
                        }
                    >
                        <Icon glyph=Glyph::Blocks class="h-6 w-6 text-primary" />
                        "MontRS"
                    </a>
                    <nav class="hidden items-center gap-1 text-sm md:flex">
                        {nav_links.into_iter().map(|(href, label)| {
                            let nav = navigate.clone();
                            view! {
                                <a
                                    href=href
                                    class="rounded-md px-3 py-2 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                                    on:click=move |ev| {
                                        ev.prevent_default();
                                        nav(href, Default::default());
                                    }
                                >{label}</a>
                            }
                        }).collect::<Vec<_>>()}
                    </nav>
                </div>

                <div class="relative flex items-center gap-2">
                    <a
                        href="https://github.com/montrs/montrs"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="hidden items-center gap-1.5 rounded-md border border-border px-2.5 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground sm:inline-flex"
                    >
                        <Icon glyph=Glyph::Star class="h-3.5 w-3.5" />
                        "Star"
                    </a>

                    // Theme toggle: defaults to System, with explicit
                    // Light/Dark choices persisted to localStorage.
                    <div class="relative">
                        <button
                            type="button"
                            class="inline-flex h-9 w-9 items-center justify-center rounded-md border border-border text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                            on:click=move |_| theme_open.update(|o| *o = !*o)
                            aria-label="Toggle theme"
                            aria-haspopup="menu"
                            aria-expanded=move || theme_open.get()
                        >
                            <Icon glyph=Signal::from(theme_icon) class="h-4 w-4" />
                        </button>

                        <Show when=move || theme_open.get()>
                            <div
                                class="fixed inset-0 z-40"
                                on:click=move |_| theme_open.set(false)
                            ></div>
                            <div
                                class="absolute right-0 z-50 mt-2 w-36 rounded-md border border-border bg-popover p-1 shadow-lg"
                                role="menu"
                                aria-label="Theme"
                            >
                                {theme_modes.into_iter().map(|(label, mode, icon)| {
                                    let mode2 = mode;
                                    let is_selected = move || theme.get() == mode2;
                                    let select = move |_| {
                                        theme.set(mode2);
                                        theme_open.set(false);
                                    };
                                    view! {
                                        <button
                                            type="button"
                                            role="menuitem"
                                            class=move || {
                                                let base = "flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-sm transition-colors";
                                                if is_selected() {
                                                    format!("{base} bg-accent font-medium text-accent-foreground")
                                                } else {
                                                    format!("{base} text-muted-foreground hover:bg-accent hover:text-accent-foreground")
                                                }
                                            }
                                            on:click=select
                                        >
                                            <Icon glyph=icon class="h-4 w-4" />
                                            {label}
                                            <span class="ml-auto flex items-center">
                                                <Icon glyph=Glyph::Check class=move || {
                                                    if is_selected() { "h-3.5 w-3.5" } else { "h-3.5 w-3.5 opacity-0" }
                                                } />
                                            </span>
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </Show>
                    </div>

                    // Mobile menu toggle
                    <button
                        type="button"
                        class="inline-flex h-9 w-9 items-center justify-center rounded-md border border-border text-muted-foreground transition-colors hover:bg-accent hover:text-foreground md:hidden"
                        on:click=move |_| mobile_open.update(|o| *o = !*o)
                        aria-label="Open menu"
                        aria-expanded=move || mobile_open.get()
                    >
                        <Icon glyph=Glyph::Menu class="h-4 w-4" />
                    </button>

                    <Show when=move || mobile_open.get()>
                        <div
                            class="fixed inset-0 z-40"
                            on:click=move |_| mobile_open.set(false)
                        ></div>
                        <div class="absolute right-0 top-12 z-50 w-44 rounded-md border border-border bg-popover p-1 shadow-lg md:hidden">
                            {nav_links.into_iter().map(|(href, label)| {
                                let nav = navigate.clone();
                                let close_menu = mobile_open;
                                view! {
                                    <a
                                        href=href
                                        class="block rounded-sm px-3 py-2 text-sm text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
                                        on:click=move |ev| {
                                            ev.prevent_default();
                                            nav(href, Default::default());
                                            close_menu.set(false);
                                        }
                                    >{label}</a>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </Show>
                </div>
            </div>
        </header>
    }
}