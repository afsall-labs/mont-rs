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

//! Site-wide theme customization (shark-ui inspired). Presets are persisted to
//! localStorage and applied as CSS variables on `<html>` so every component
//! re-themes live. "Copy theme" emits a `:root` CSS snippet for your project.

use leptos::prelude::*;
use montrs_icons::*;
use montrs_ui::prelude::*;

/// (label, hsl triplet, foreground hsl)
const PRIMARY_OPTIONS: &[(&str, &str, &str)] = &[
    ("Rust", "24.6 94.8% 53.1%", "0 0% 100%"),
    ("Red", "0 84% 60%", "0 0% 100%"),
    ("Orange", "24 95% 53%", "0 0% 100%"),
    ("Amber", "38 92% 50%", "0 0% 100%"),
    ("Lime", "84 81% 44%", "120 100% 4%"),
    ("Green", "142 71% 45%", "0 0% 100%"),
    ("Teal", "172 66% 40%", "0 0% 100%"),
    ("Blue", "217 91% 60%", "0 0% 100%"),
    ("Violet", "258 90% 66%", "0 0% 100%"),
    ("Pink", "330 81% 60%", "0 0% 100%"),
];

/// (label, background hsl, foreground hsl, muted-foreground hsl, border hsl)
const GRAY_OPTIONS: &[(&str, &str, &str, &str, &str)] = &[
    ("Near Black", "0 0% 4%", "0 0% 98%", "240 5% 65%", "0 0% 12%"),
    ("Zinc", "240 10% 4%", "240 6% 98%", "240 5% 65%", "240 4% 16%"),
    ("Slate", "240 6% 4%", "210 20% 98%", "215 16% 65%", "215 16% 14%"),
    ("Stone", "24 10% 4%", "60 8% 98%", "24 6% 64%", "20 6% 14%"),
    ("Neutral", "0 0% 9%", "0 0% 98%", "0 0% 65%", "0 0% 20%"),
];

const RADIUS_OPTIONS: &[(&str, &str)] = &[
    ("Sharp", "0rem"),
    ("Small", "0.25rem"),
    ("Default", "0.5rem"),
    ("Large", "0.75rem"),
    ("Pill", "1rem"),
];

#[allow(dead_code)]
const STORAGE_KEY: &str = "montrs-theme-config";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ThemeCfg {
    primary: usize, // index into PRIMARY_OPTIONS
    gray: usize,    // index into GRAY_OPTIONS
    radius: usize,  // index into RADIUS_OPTIONS
}

fn load_cfg() -> ThemeCfg {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
            && let Ok(Some(raw)) = storage.get_item(STORAGE_KEY)
            && let Some((p, g, r)) = raw.split_once(',').and_then(|(a, rest)| {
                rest.split_once(',').map(|(b, c)| (a, b, c))
            }) && let (Ok(p), Ok(g), Ok(r)) =
                (p.parse::<usize>(), g.parse::<usize>(), r.parse::<usize>())
        {
            return ThemeCfg {
                primary: p.min(PRIMARY_OPTIONS.len() - 1),
                gray: g.min(GRAY_OPTIONS.len() - 1),
                radius: r.min(RADIUS_OPTIONS.len() - 1),
            };
        }
    }
    ThemeCfg {
        primary: 0,
        gray: 0,
        radius: 2,
    }
}

#[allow(unused_variables)]
fn save_cfg(cfg: ThemeCfg) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
        {
            let _ = storage.set_item(
                STORAGE_KEY,
                &format!("{},{},{}", cfg.primary, cfg.gray, cfg.radius),
            );
        }
    }
}

#[allow(unused_variables)]
fn apply_cfg(cfg: ThemeCfg) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        if let Some(document) = web_sys::window().and_then(|w| w.document())
            && let Some(doc_el) = document.document_element()
            && let Some(html) = doc_el.dyn_ref::<web_sys::HtmlElement>()
        {
            let (_, primary, primary_fg) = PRIMARY_OPTIONS[cfg.primary];
            let (_, bg, fg, muted_fg, border) = GRAY_OPTIONS[cfg.gray];
            let (_, radius) = RADIUS_OPTIONS[cfg.radius];
            let s = html.style();
            let _ = s.set_property("--primary", primary);
            let _ = s.set_property("--primary-foreground", primary_fg);
            let _ = s.set_property("--ring", primary);
            let _ = s.set_property("--background", bg);
            let _ = s.set_property("--foreground", fg);
            let _ = s.set_property("--muted-foreground", muted_fg);
            let _ = s.set_property("--border", border);
            let _ = s.set_property("--input", border);
            let _ = s.set_property("--radius", radius);
        }
    }
}

fn copy_css(cfg: ThemeCfg) -> String {
    let (_, primary, primary_fg) = PRIMARY_OPTIONS[cfg.primary];
    let (_, bg, fg, muted_fg, border) = GRAY_OPTIONS[cfg.gray];
    let (radius_label, radius) = RADIUS_OPTIONS[cfg.radius];
    format!(
        "/* {radius_label} radius · primary {primary} */\n\
         :root {{\n  --radius: {radius};\n  --background: {bg};\n  \
         --foreground: {fg};\n  --muted-foreground: {muted_fg};\n  \
         --border: {border};\n  --input: {border};\n  \
         --primary: {primary};\n  --primary-foreground: {primary_fg};\n  \
         --ring: {primary};\n}}\n"
    )
}

#[component]
pub fn ThemeCustomizer() -> impl IntoView {
    let cfg = RwSignal::new(load_cfg());
    let copied = RwSignal::new(false);

    Effect::new(move |_| {
        let c = cfg.get();
        apply_cfg(c);
        save_cfg(c);
    });

    let reset = move |_| {
        cfg.set(ThemeCfg {
            primary: 0,
            gray: 0,
            radius: 2,
        });
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            if let Some(document) = web_sys::window().and_then(|w| w.document())
                && let Some(doc_el) = document.document_element()
                && let Some(html) = doc_el.dyn_ref::<web_sys::HtmlElement>()
            {
                let s = html.style();
                for prop in [
                    "--primary",
                    "--primary-foreground",
                    "--ring",
                    "--background",
                    "--foreground",
                    "--muted-foreground",
                    "--border",
                    "--input",
                    "--radius",
                ] {
                    let _ = s.remove_property(prop);
                }
            }
        }
    };

    view! {
        <div class="space-y-6">
            <div>
                <p class="mb-2 font-mono text-[11px] uppercase tracking-wide text-muted-foreground">
                    "Primary color"
                </p>
                <div class="grid grid-cols-5 gap-2">
                    {PRIMARY_OPTIONS.iter().enumerate().map(|(i, (label, hsl, _))| {
                        let i2 = i;
                        let is_active = move || cfg.get().primary == i2;
                        let swatch = format!("hsl({hsl})");
                        view! {
                            <button
                                type="button"
                                class=move || {
                                    let base = "flex flex-col items-center gap-1 rounded-md border p-2 transition-colors";
                                    if is_active() {
                                        format!("{base} border-primary")
                                    } else {
                                        format!("{base} border-border hover:border-ring/50")
                                    }
                                }
                                on:click=move |_| cfg.update(|c| c.primary = i2)
                            >
                                <span class="h-5 w-5 rounded-full border border-border" style=format!("background-color: {swatch};")></span>
                                <span class="text-[10px] text-muted-foreground">{*label}</span>
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            <div>
                <p class="mb-2 font-mono text-[11px] uppercase tracking-wide text-muted-foreground">
                    "Background"
                </p>
                <div class="grid grid-cols-5 gap-2">
                    {GRAY_OPTIONS.iter().enumerate().map(|(i, (label, bg, fg, _, _))| {
                        let i2 = i;
                        let is_active = move || cfg.get().gray == i2;
                        let swatch = format!("hsl({bg})");
                        let fg2 = format!("hsl({fg})");
                        view! {
                            <button
                                type="button"
                                class=move || {
                                    let base = "flex flex-col items-center gap-1 rounded-md border p-2 transition-colors";
                                    if is_active() {
                                        format!("{base} border-primary")
                                    } else {
                                        format!("{base} border-border hover:border-ring/50")
                                    }
                                }
                                on:click=move |_| cfg.update(|c| c.gray = i2)
                            >
                                <span class="h-5 w-5 rounded-full border border-border" style=format!("background-color: {swatch}; color: {fg2};")></span>
                                <span class="text-[10px] text-muted-foreground">{*label}</span>
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            <div>
                <p class="mb-2 font-mono text-[11px] uppercase tracking-wide text-muted-foreground">
                    "Border radius"
                </p>
                <div class="flex flex-wrap gap-2">
                    {RADIUS_OPTIONS.iter().enumerate().map(|(i, (label, _))| {
                        let i2 = i;
                        let is_active = move || cfg.get().radius == i2;
                        view! {
                            <button
                                type="button"
                                class=move || {
                                    let base = "rounded-full border px-3 py-1 text-xs font-medium transition-colors";
                                    if is_active() {
                                        format!("{base} border-primary bg-primary/10 text-primary")
                                    } else {
                                        format!("{base} border-border text-muted-foreground hover:bg-accent hover:text-foreground")
                                    }
                                }
                                on:click=move |_| cfg.update(|c| c.radius = i2)
                            >{*label}</button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            <div class="flex flex-wrap items-center gap-2 border-t border-border pt-4">
                <button
                    type="button"
                    class=move || {
                        let base = "copy-btn inline-flex items-center gap-1";
                        if copied.get() {
                            format!("{base} border-transparent bg-primary/15 text-primary")
                        } else {
                            base.to_string()
                        }
                    }
                    on:click={
                        let cfg = cfg;
                        move |_| {
                            crate::copy::copy_text(&copy_css(cfg.get()));
                            copied.set(true);
                            #[cfg(target_arch = "wasm32")]
                            {
                                use wasm_bindgen::prelude::*;
                                let c2 = copied;
                                let cb = wasm_bindgen::prelude::Closure::wrap(Box::new(
                                    move || c2.set(false),
                                ) as Box<dyn FnMut()>);
                                if let Some(window) = web_sys::window() {
                                    let _ = window
                                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                                            cb.as_ref().unchecked_ref(),
                                            1500,
                                        );
                                }
                                cb.forget();
                            }
                        }
                    }
                >
                    {move || if copied.get() { "Copied".to_string() } else { "Copy theme CSS".to_string() }}
                </button>
                <button
                    type="button"
                    class="inline-flex items-center gap-1.5 rounded-md border border-border px-3 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                    on:click=reset
                >
                    <Icon glyph=Glyph::RotateCcw class="h-3.5 w-3.5" />
                    "Reset"
                </button>
            </div>
        </div>
    }
}