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
use montrs_core::nav::*;
use montrs_icons::*;
use montrs_ui::prelude::*;

fn icon_size_class(size: &str) -> &'static str {
    match size {
        "sm" => "w-4 h-4",
        "lg" => "w-8 h-8",
        "xl" => "w-12 h-12",
        "2xl" => "w-16 h-16",
        _ => "w-6 h-6",
    }
}

fn formatted_name(name: &str) -> String {
    name.split('-')
        .map(|part| {
            let mut c = part.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[component]
pub fn Icons() -> impl IntoView {
    let query = use_query_map();
    let navigate = use_navigate();
    let search = RwSignal::new(query.get().get("search").unwrap_or_default());
    let size = RwSignal::new(query.get().get("size").unwrap_or_default());
    let stroke_w = RwSignal::new(query.get().get("stroke").unwrap_or_default());
    let style_mode = RwSignal::new(
        query
            .get()
            .get("style")
            .unwrap_or_else(|| "stroke".to_string()),
    );
    let stroke_color =
        RwSignal::new(query.get().get("stroke_color").unwrap_or_default());
    let fill_color =
        RwSignal::new(query.get().get("fill_color").unwrap_or_default());
    let animated = RwSignal::new(false);

    let icons = Memo::new(move |_| {
        let s = search.get();
        if s.is_empty() {
            Glyph::find("")
        } else {
            Glyph::find(&s)
        }
    });

    let selected_icon = RwSignal::new(None::<Glyph>);

    let size_class = move || icon_size_class(&size.get());

    // Effective stroke / fill colors based on the selected style mode.
    let stroke_value = Signal::derive(move || {
        let mode = style_mode.get();
        if mode == "fill" {
            return "none".to_string();
        }
        let c = stroke_color.get();
        if c.is_empty() {
            "currentColor".to_string()
        } else {
            c
        }
    });

    let fill_value = Signal::derive(move || {
        let mode = style_mode.get();
        if mode == "stroke" {
            return "none".to_string();
        }
        let c = fill_color.get();
        if c.is_empty() {
            "currentColor".to_string()
        } else {
            c
        }
    });

    let stroke_width_val = Signal::derive(move || {
        let s = stroke_w.get();
        if s.is_empty() { "1.5".to_string() } else { s }
    });

    // Handlers are defined at the top level (outside the view) so they can be
    // captured freely by <Show>/<For> children without Fn-vs-FnOnce issues.
    let on_search_input = {
        let nav = navigate.clone();
        move |e: leptos::ev::Event| {
            let val = event_target_value(&e);
            search.set(val.clone());
            nav(&format!("/ui/icons?search={}", val), Default::default());
        }
    };
    let on_size_change = {
        let nav = navigate.clone();
        move |e: leptos::ev::Event| {
            let val = event_target_value(&e);
            size.set(val.clone());
            nav(&format!("/ui/icons?size={}", val), Default::default());
        }
    };
    let on_stroke_w_change = {
        let nav = navigate.clone();
        move |e: leptos::ev::Event| {
            let val = event_target_value(&e);
            stroke_w.set(val.clone());
            nav(&format!("/ui/icons?stroke={}", val), Default::default());
        }
    };
    let on_style_change = {
        let nav = navigate.clone();
        move |e: leptos::ev::Event| {
            let val = event_target_value(&e);
            style_mode.set(val.clone());
            nav(&format!("/ui/icons?style={}", val), Default::default());
        }
    };
    let on_stroke_color = {
        let nav = navigate.clone();
        move |e: leptos::ev::Event| {
            let val = event_target_value(&e);
            stroke_color.set(val.clone());
            nav(
                &format!("/ui/icons?stroke_color={}", val),
                Default::default(),
            );
        }
    };
    let on_fill_color = {
        let nav = navigate.clone();
        move |e: leptos::ev::Event| {
            let val = event_target_value(&e);
            fill_color.set(val.clone());
            nav(&format!("/ui/icons?fill_color={}", val), Default::default());
        }
    };
    let on_reset = {
        let nav = navigate.clone();
        move |_: leptos::ev::MouseEvent| {
            search.set(String::new());
            size.set(String::new());
            stroke_w.set(String::new());
            style_mode.set("stroke".to_string());
            stroke_color.set(String::new());
            fill_color.set(String::new());
            nav("/ui/icons", Default::default());
        }
    };

    let stroke_disabled = move || style_mode.get() == "fill";
    let fill_disabled = move || style_mode.get() == "stroke";

    view! {
        <div class="page-container py-12">
            <div class="mb-8 flex flex-wrap items-end justify-between gap-4">
                <div>
                    <h1 class="text-3xl font-bold tracking-tight">"Icons"</h1>
                    <p class="mt-2 text-muted-foreground">
                        {move || format!("{} icons — click one to copy", icons.get().len())}
                    </p>
                </div>
                <label class="inline-flex items-center gap-2 text-sm">
                    "Animated"
                    <button
                        type="button"
                        role="switch"
                        aria-checked=animated
                        class=move || {
                            let base = "relative h-6 w-11 rounded-full border-2 border-transparent transition-colors";
                            if animated.get() { format!("{base} bg-primary") } else { format!("{base} bg-input") }
                        }
                        on:click=move |_| animated.update(|v| *v = !*v)
                    >
                        <span class=move || {
                            let base = "pointer-events-none block h-5 w-5 rounded-full bg-background shadow transition-transform";
                            if animated.get() { format!("{base} translate-x-5") } else { format!("{base} translate-x-0") }
                        }></span>
                    </button>
                    <span class="text-xs text-muted-foreground">
                        {move || if animated.get() { "hover to play" } else { "static" }}
                    </span>
                </label>
            </div>

            <div class="mb-8 flex flex-wrap items-center gap-3">
                <div class="relative flex-1 min-w-[220px]">
                    <Icon glyph=Glyph::Search class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <input
                        type="search"
                        placeholder="Search icons…"
                        class="h-10 w-full rounded-md border border-input bg-background pl-9 pr-3 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        prop:value=search
                        on:input=on_search_input
                    />
                </div>

                <select
                    class="h-10 rounded-md border border-input bg-background px-3 text-sm"
                    prop:value=size
                    on:change=on_size_change
                >
                    <option value="">"Size: default"</option>
                    <option value="sm">"Small"</option>
                    <option value="lg">"Large"</option>
                    <option value="xl">"X-large"</option>
                    <option value="2xl">"2x-large"</option>
                </select>

                <select
                    class="h-10 rounded-md border border-input bg-background px-3 text-sm"
                    prop:value=stroke_w
                    on:change=on_stroke_w_change
                >
                    <option value="">"Stroke: 1.5"</option>
                    <option value="1">"Stroke: 1"</option>
                    <option value="2">"Stroke: 2"</option>
                    <option value="2.5">"Stroke: 2.5"</option>
                    <option value="3">"Stroke: 3"</option>
                </select>

                <select
                    class="h-10 rounded-md border border-input bg-background px-3 text-sm"
                    prop:value=style_mode
                    on:change=on_style_change
                >
                    <option value="stroke">"Style: stroke"</option>
                    <option value="fill">"Style: fill"</option>
                    <option value="both">"Style: stroke + fill"</option>
                </select>

                <label class=move || {
                        let base = "inline-flex h-10 items-center gap-2 rounded-md border border-input bg-background px-3 text-sm transition-opacity";
                        if stroke_disabled() { format!("{base} opacity-50") } else { base.to_string() }
                    }>
                    "Stroke"
                    <input
                        type="color"
                        class="h-6 w-8 cursor-pointer rounded border border-border bg-transparent"
                        prop:value=stroke_color
                        on:input=on_stroke_color
                        disabled=stroke_disabled
                    />
                </label>

                <label class=move || {
                        let base = "inline-flex h-10 items-center gap-2 rounded-md border border-input bg-background px-3 text-sm transition-opacity";
                        if fill_disabled() { format!("{base} opacity-50") } else { base.to_string() }
                    }>
                    "Fill"
                    <input
                        type="color"
                        class="h-6 w-8 cursor-pointer rounded border border-border bg-transparent"
                        prop:value=fill_color
                        on:input=on_fill_color
                        disabled=fill_disabled
                    />
                </label>

                <button
                    type="button"
                    class="inline-flex h-10 items-center rounded-md border border-border px-3 text-sm text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                    on:click=on_reset
                >
                    "Reset"
                </button>
            </div>

            <div class="grid grid-cols-4 gap-2 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 xl:grid-cols-12">
                <For
                    each=move || icons.get()
                    key=|g| *g
                    children=move |glyph| {
                        let name = glyph.name().to_string();
                        let kebab = glyph.kebab_name();
                        let selected = selected_icon;
                        let is_animated = animated;
                        view! {
                            <button
                                type="button"
                                class="flex flex-col items-center gap-2 rounded-lg border border-border p-3 transition-colors hover:border-ring/40 hover:bg-accent"
                                on:click=move |_| selected.set(Some(glyph))
                                title=name.clone()
                            >
                                <Show
                                    when=move || is_animated.get()
                                    fallback=move || view! {
                                        <Icon
                                            glyph=Signal::from(glyph)
                                            class=size_class
                                            stroke=stroke_value
                                            stroke_width=stroke_width_val
                                            fill=fill_value
                                        />
                                    }
                                >
                                    <AnimatedIcon
                                        glyph=Signal::from(glyph)
                                        class=size_class
                                        stroke=stroke_value
                                        stroke_width=stroke_width_val
                                        fill=fill_value
                                    />
                                </Show>
                                <span class="w-full truncate text-center font-mono text-[10px] text-muted-foreground">{kebab}</span>
                            </button>
                        }
                    }
                />
            </div>

            {move || selected_icon.get().map(|glyph| {
                let name = glyph.name().to_string();
                let kebab = glyph.kebab_name().to_string();
                let svg = glyph.svg().to_string();
                let animated_state = animated;
                let usage = {
                    let a = animated_state.get();
                    let mode = style_mode.get();
                    let sw = stroke_w.get();
                    let mut props = String::from("class=\"w-6 h-6\"");
                    if !sw.is_empty() {
                        props.push_str(&format!(" stroke_width=\"{}\"", sw));
                    }
                    if mode != "stroke" {
                        let fc = fill_color.get();
                        if !fc.is_empty() {
                            props.push_str(&format!(" fill=\"{}\"", fc));
                        }
                    }
                    if a {
                        format!(r#"<AnimatedIcon glyph=Glyph::{name} {props} />"#)
                    } else {
                        format!(r#"<Icon glyph=Glyph::{name} {props} />"#)
                    }
                };
                let import_statement = "use montrs_icons::Glyph;".to_string();
                let mode_for_icon = move || {
                    let m = style_mode.get();
                    let sc = stroke_color.get();
                    let fc = fill_color.get();
                    let mut css = String::new();
                    if m != "fill" {
                        if !sc.is_empty() {
                            css.push_str(&format!("color: {sc};"));
                        } else {
                            css.push_str("color: currentColor;");
                        }
                    }
                    if m != "stroke" && !fc.is_empty() {
                        css.push_str(&format!(" fill: {fc};"));
                    }
                    css
                };
                view! {
                    <div
                        class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 p-4 backdrop-blur-sm"
                        on:click=move |_| selected_icon.set(None)
                    >
                        <div
                            class="w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg"
                            on:click=|e| { e.stop_propagation(); }
                        >
                            <div class="flex items-start justify-between gap-4">
                                <div
                                    class="flex h-16 w-16 items-center justify-center rounded-lg border border-border bg-background"
                                    style=mode_for_icon
                                >
                                    <Show
                                        when=move || animated_state.get()
                                        fallback=move || view! {
                                            <Icon
                                                glyph=Signal::from(glyph)
                                                class="h-8 w-8"
                                                stroke=stroke_value
                                                stroke_width=stroke_width_val
                                                fill=fill_value
                                            />
                                        }
                                    >
                                        <AnimatedIcon
                                            glyph=Signal::from(glyph)
                                            class="h-8 w-8"
                                            stroke=stroke_value
                                            stroke_width=stroke_width_val
                                            fill=fill_value
                                        />
                                    </Show>
                                </div>
                                <button
                                    type="button"
                                    class="rounded-md p-2 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                                    on:click=move |_| selected_icon.set(None)
                                    aria-label="Close"
                                >
                                    <Icon glyph=Glyph::X class="h-4 w-4" />
                                </button>
                            </div>

                            <h3 class="mt-4 text-lg font-semibold">{formatted_name(&kebab)}</h3>
                            <p class="font-mono text-xs text-muted-foreground">{kebab.clone()}</p>

                            <div class="mt-4 space-y-2">
                                <div class="flex items-center gap-2 rounded-md border border-border bg-background p-2">
                                    <code class="flex-1 truncate text-xs">{usage.clone()}</code>
                                    <CopyButton text=usage.clone() label="Copy" />
                                </div>
                                <div>
                                    <p class="mb-1 font-mono text-[11px] uppercase tracking-wide text-muted-foreground">"SVG"</p>
                                    <div class="flex items-center gap-2 rounded-md border border-border bg-background p-2">
                                        <code class="max-h-24 flex-1 overflow-y-auto text-[10px] break-all">{svg.clone()}</code>
                                        <CopyButton text=svg.clone() label="Copy" />
                                    </div>
                                </div>
                                <div class="flex items-center gap-2 rounded-md border border-border bg-background p-2">
                                    <code class="flex-1 truncate text-xs">{import_statement.clone()}</code>
                                    <CopyButton text=import_statement label="Copy" />
                                </div>
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
