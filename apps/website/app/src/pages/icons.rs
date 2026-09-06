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
use montrs_icons::collections::CollectedGlyph;
use montrs_icons::{AnimatedSvg, Collection, Glyph, Icon};
use montrs_ui::prelude::*;

const PAGE_SIZE: usize = 200;

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

fn full_svg_markup(g: &CollectedGlyph, size: u32, stroke_w: f64) -> String {
    let sw = if g.stroke == "none" {
        String::new()
    } else {
        format!(" stroke-width=\"{stroke_w}\"")
    };
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" \
         height=\"{size}\" viewBox=\"{}\" fill=\"{}\" stroke=\"{}\"{sw} \
         stroke-linecap=\"round\" stroke-linejoin=\"round\">{}</svg>",
        g.viewbox, g.fill, g.stroke, g.svg
    )
}

// ---------------------------------------------------------------------------
// MRU (localStorage, client-only, works for every collection)
// ---------------------------------------------------------------------------

fn load_mru() -> Vec<(Collection, String)> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
            && let Ok(Some(raw)) = storage.get_item("montrs-icons-mru")
            && let Ok(items) = serde_json::from_str::<Vec<(String, String)>>(&raw)
        {
            return items
                .into_iter()
                .filter_map(|(col, name)| {
                    Collection::from_key(&col).map(|c| (c, name))
                })
                .collect::<Vec<_>>();
        }
    }
    Vec::new()
}

#[allow(unused_variables)]
fn save_mru(items: &[(Collection, String)]) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let data: Vec<(String, String)> = items
                .iter()
                .map(|(c, n)| (c.key().to_string(), n.clone()))
                .collect();
            if let Ok(json) = serde_json::to_string(&data)
                && let Ok(Some(storage)) = window.local_storage()
            {
                let _ = storage.set_item("montrs-icons-mru", &json);
            }
        }
    }
}

#[component]
pub fn Icons() -> impl IntoView {
    let query = use_query_map();
    let navigate = use_navigate();

    let collection = RwSignal::new(
        query
            .get()
            .get("collection")
            .and_then(|k| Collection::from_key(&k))
            .unwrap_or(Collection::Lucide),
    );
    let search = RwSignal::new(query.get().get("q").unwrap_or_default());
    let size_px = RwSignal::new(
        query
            .get()
            .get("size")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(24),
    );
    let stroke_w = RwSignal::new(
        query
            .get()
            .get("sw")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1.5),
    );
    let color = RwSignal::new(query.get().get("color").unwrap_or_default());
    let category = RwSignal::new(query.get().get("cat").unwrap_or_default());
    let animated = RwSignal::new(
        query.get().get("anim").is_none_or(|v| v != "0"),
    );
    let page = RwSignal::new(1usize);

    let hydrated = RwSignal::new(false);
    let mru = RwSignal::new(Vec::<(Collection, String)>::new());
    let selected_icon = RwSignal::new(None::<CollectedGlyph>);
    let anim_choice = RwSignal::new("auto".to_string());

    Effect::new(move |_| {
        if !hydrated.get() {
            hydrated.set(true);
            mru.set(load_mru());
        }
    });

    // Reset pagination whenever filters or the collection change.
    Effect::new(move |_| {
        search.get();
        category.get();
        collection.get();
        page.set(1);
    });

    let is_lucide = move || collection.get() == Collection::Lucide;

    let filtered = Memo::new(move |_| {
        let s = search.get().to_lowercase();
        let cat = category.get();
        if collection.get() == Collection::Lucide {
            let mut found = if s.is_empty() {
                Glyph::find("")
            } else {
                Glyph::find(&s)
            };
            if !cat.is_empty() {
                found.retain(|g| {
                    g.categories().any(|c| c.eq_ignore_ascii_case(&cat))
                });
            }
            found
                .into_iter()
                .map(|g| CollectedGlyph {
                    name: g.name(),
                    svg: g.svg(),
                    viewbox: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                })
                .collect::<Vec<_>>()
        } else {
            collection
                .get()
                .icons()
                .into_iter()
                .filter(|g| s.is_empty() || g.name.to_lowercase().contains(&s))
                .collect::<Vec<_>>()
        }
    });

    let total_pages = Memo::new(move |_| {
        let len = filtered.get().len();
        if len == 0 {
            1
        } else {
            (len + PAGE_SIZE - 1) / PAGE_SIZE
        }
    });

    let page_icons = Memo::new(move |_| {
        let all = filtered.get();
        let p = page.get().min(total_pages.get()).max(1);
        let start = (p - 1) * PAGE_SIZE;
        all.into_iter()
            .skip(start)
            .take(PAGE_SIZE)
            .collect::<Vec<_>>()
    });

    let categories = Glyph::all_categories();

    let select_icon = move |glyph: CollectedGlyph| {
        selected_icon.set(Some(glyph));
        anim_choice.set("auto".to_string());
        mru.update(|v| {
            v.retain(|(_, n)| *n != glyph.name);
            v.insert(0, (collection.get(), glyph.name.to_string()));
            v.truncate(8);
            save_mru(v);
        });
    };

    let clear_filters = move |_: leptos::ev::MouseEvent| {
        search.set(String::new());
        category.set(String::new());
    };

    let sync_url = {
        let nav = navigate.clone();
        move || {
            let mut q = format!(
                "/ui/icons?collection={}&size={}&sw={}",
                collection.get().key(),
                size_px.get(),
                stroke_w.get()
            );
            let s = search.get();
            if !s.is_empty() {
                q.push_str(&format!("&q={}", s));
            }
            let c = color.get();
            if !c.is_empty() {
                q.push_str(&format!("&color={}", c));
            }
            let cat = category.get();
            if !cat.is_empty() {
                q.push_str(&format!("&cat={}", cat));
            }
            if !animated.get() {
                q.push_str("&anim=0");
            }
            nav(
                &q,
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        }
    };

    let on_search = {
        let sync = sync_url.clone();
        move |e: leptos::ev::Event| {
            search.set(event_target_value(&e));
            sync();
        }
    };
    let on_size = {
        let sync = sync_url.clone();
        move |e: leptos::ev::Event| {
            if let Ok(v) = event_target_value(&e).parse::<u32>() {
                size_px.set(v.clamp(14, 48));
            }
            sync();
        }
    };
    let on_stroke_w = {
        let sync = sync_url.clone();
        move |e: leptos::ev::Event| {
            if let Ok(v) = event_target_value(&e).parse::<f64>() {
                stroke_w.set(v.clamp(0.5, 3.0));
            }
            sync();
        }
    };
    let on_color = {
        let sync = sync_url.clone();
        move |e: leptos::ev::Event| {
            color.set(event_target_value(&e));
            sync();
        }
    };
    let on_color_reset = {
        let sync = sync_url.clone();
        move |_: leptos::ev::MouseEvent| {
            color.set(String::new());
            sync();
        }
    };
    let on_size_input = {
        let sync = sync_url.clone();
        move |e: leptos::ev::Event| {
            let val = event_target_value(&e);
            if let Ok(v) = val.parse::<u32>() {
                size_px.set(v.clamp(14, 48));
            }
            sync();
        }
    };
    let on_stroke_input = {
        let sync = sync_url.clone();
        move |e: leptos::ev::Event| {
            let val = event_target_value(&e);
            if let Ok(v) = val.parse::<f64>() {
                stroke_w.set(v.clamp(0.5, 3.0));
            }
            sync();
        }
    };

    let stroke_val = Signal::derive(move || {
        let c = color.get();
        if c.is_empty() { "currentColor".to_string() } else { c }
    });
    let size_val = Signal::derive(move || size_px.get().to_string());
    let sw_val = Signal::derive(move || format!("{:.2}", stroke_w.get()));
    let mru_visible =
        move || search.get().is_empty() && category.get().is_empty();

    let anim_choices = [
        ("auto", "Auto"),
        ("draw", "Draw"),
        ("spin", "Spin"),
        ("pulse", "Pulse"),
        ("bounce", "Bounce"),
        ("ping", "Ping"),
        ("shake", "Shake"),
        ("nod", "Nod"),
        ("off", "Off"),
    ];

    view! {
        <div class="flex">
            // ---------------------------------------------------------------
            // Sidebar
            // ---------------------------------------------------------------
            <aside class="icons-sidebar hidden lg:block">
                <div class="sticky top-16 max-h-[calc(100vh-4rem)] overflow-y-auto">
                    <div class="icons-sidebar-section">
                        <div class="flex items-center gap-2">
                            <img src="/logo-64.png" alt="MontRS" class="h-8 w-8 rounded" />
                            <div>
                                <p class="text-sm font-semibold">"Icons"</p>
                                <p class="font-mono text-[11px] text-muted-foreground">
                                    {move || format!("{} icons", filtered.get().len())}
                                </p>
                            </div>
                        </div>
                    </div>

                    <div class="icons-sidebar-section">
                        <p class="icons-sidebar-heading">"Collection"</p>
                        <div class="space-y-0.5">
                            {Collection::ALL.iter().map(|c| {
                                let c = *c;
                                let label = c.label().to_string();
                                let count = c.count();
                                let is_active = move || collection.get() == c;
                                view! {
                                    <button
                                        type="button"
                                        class=move || {
                                            let base = "flex w-full items-center justify-between rounded-md px-2 py-1 text-left text-sm transition-colors";
                                            if is_active() {
                                                format!("{base} bg-accent font-medium text-foreground")
                                            } else {
                                                format!("{base} text-muted-foreground hover:bg-accent/60 hover:text-foreground")
                                            }
                                        }
                                        on:click={
                                            let sync = sync_url.clone();
                                            move |_| { collection.set(c); category.set(String::new()); sync(); }
                                        }
                                    >
                                        <span>{label}</span>
                                        <span class="font-mono text-[10px]">{count.to_string()}</span>
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>

                    <div class="icons-sidebar-section">
                        <p class="icons-sidebar-heading">"Customize"</p>
                        <div class="space-y-4">
                            <label class="block">
                                <span class="flex items-center justify-between text-xs text-muted-foreground">
                                    "Size"
                                    <input
                                        type="text"
                                        inputmode="numeric"
                                        class="h-6 w-16 rounded border border-border bg-background px-1 text-center font-mono text-xs text-foreground"
                                        prop:value=move || size_px.get().to_string()
                                        on:change=on_size_input
                                        title="14–48"
                                    />
                                </span>
                                <input
                                    type="range"
                                    min="14"
                                    max="48"
                                    step="1"
                                    class="icon-range mt-1"
                                    prop:value=move || size_px.get().to_string()
                                    on:input=on_size
                                />
                            </label>
                            <label class="block">
                                <span class="flex items-center justify-between text-xs text-muted-foreground">
                                    "Stroke width"
                                    <input
                                        type="text"
                                        inputmode="decimal"
                                        class="h-6 w-16 rounded border border-border bg-background px-1 text-center font-mono text-xs text-foreground"
                                        prop:value=move || format!("{:.2}", stroke_w.get())
                                        on:change=on_stroke_input
                                        title="0.5–3"
                                    />
                                </span>
                                <input
                                    type="range"
                                    min="0.5"
                                    max="3"
                                    step="0.25"
                                    class="icon-range mt-1"
                                    prop:value=move || stroke_w.get().to_string()
                                    on:input=on_stroke_w
                                />
                            </label>
                            <label class="block">
                                <span class="mb-1 flex items-center justify-between text-xs text-muted-foreground">
                                    "Color"
                                    <button
                                        type="button"
                                        class="text-[10px] text-muted-foreground underline-offset-2 hover:text-foreground hover:underline"
                                        on:click=on_color_reset
                                    >
                                        "reset"
                                    </button>
                                </span>
                                <input
                                    type="text"
                                    spellcheck="false"
                                    placeholder="#f97316"
                                    class="h-8 w-full rounded-md border border-input bg-background px-2 font-mono text-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                                    prop:value=color
                                    on:input=on_color
                                />
                                <p class="mt-1 text-[10px] text-muted-foreground">
                                    "hex, rgb() or hsl()"
                                </p>
                            </label>
                            <label class="flex items-center justify-between text-xs text-muted-foreground">
                                "Animated"
                                <button
                                    type="button"
                                    role="switch"
                                    aria-checked=animated
                                    class=move || {
                                        let base = "relative h-6 w-11 rounded-full border transition-colors";
                                        if animated.get() { format!("{base} border-primary bg-primary/30") } else { format!("{base} border-border bg-muted") }
                                    }
                                    on:click={
                                        let sync = sync_url.clone();
                                        move |_| { animated.update(|v| *v = !*v); sync(); }
                                    }
                                >
                                    <span class=move || {
                                        let base = "pointer-events-none absolute top-1/2 h-4 w-4 -translate-y-1/2 rounded-full bg-primary shadow transition-all";
                                        if animated.get() { format!("{base} left-6") } else { format!("{base} left-1 bg-muted-foreground") }
                                    }></span>
                                </button>
                            </label>
                            <Show when=move || !search.get().is_empty() || !category.get().is_empty()>
                                <button
                                    type="button"
                                    class="w-full rounded-md border border-border px-3 py-1.5 text-xs font-medium transition-colors hover:bg-accent"
                                    on:click=clear_filters
                                >
                                    "Clear filters"
                                </button>
                            </Show>
                        </div>
                    </div>

                    <Show when=is_lucide>
                        <div class="icons-sidebar-section">
                            <p class="icons-sidebar-heading">"Categories"</p>
                            <div class="max-h-64 space-y-0.5 overflow-y-auto pr-1">
                                <button
                                    type="button"
                                    class=move || {
                                        let base = "flex w-full items-center justify-between rounded-md px-2 py-1 text-left text-sm transition-colors";
                                        if category.get().is_empty() {
                                            format!("{base} bg-accent font-medium text-foreground")
                                        } else {
                                            format!("{base} text-muted-foreground hover:bg-accent/60 hover:text-foreground")
                                        }
                                    }
                                    on:click={
                                        let sync = sync_url.clone();
                                        move |_| { category.set(String::new()); sync(); }
                                    }
                                >
                                    <span>"All"</span>
                                    <span class="font-mono text-[10px]">{Glyph::count().to_string()}</span>
                                </button>
                                {categories.iter().map(|(title, count)| {
                                    let cat = title.clone();
                                    let title2 = title.clone();
                                    let count2 = count.to_string();
                                    let cat_for_active = cat.clone();
                                    let cat_for_click = cat.clone();
                                    let is_active = move || category.get().eq_ignore_ascii_case(&cat_for_active);
                                    let sync = sync_url.clone();
                                    view! {
                                        <button
                                            type="button"
                                            class=move || {
                                                let base = "flex w-full items-center justify-between rounded-md px-2 py-1 text-left text-sm transition-colors";
                                                if is_active() {
                                                    format!("{base} bg-accent font-medium text-foreground")
                                                } else {
                                                    format!("{base} text-muted-foreground hover:bg-accent/60 hover:text-foreground")
                                                }
                                            }
                                            on:click=move |_| { category.set(cat_for_click.clone()); sync(); }
                                        >
                                            <span>{title2}</span>
                                            <span class="font-mono text-[10px]">{count2}</span>
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    </Show>
                </div>
            </aside>

            // ---------------------------------------------------------------
            // Main column
            // ---------------------------------------------------------------
            <div class="min-w-0 flex-1 px-6 py-8">
                <div class="mb-6 flex flex-wrap items-center justify-between gap-4">
                    <div>
                        <h1 class="text-2xl font-bold tracking-tight">"Icons"</h1>
                        <p class="mt-1 text-sm text-muted-foreground">
                            {move || format!("{} shown · hover to play", page_icons.get().len())}
                        </p>
                    </div>
                    <div class="relative w-full max-w-sm">
                        <Icon glyph=Glyph::Search class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                        <input
                            type="search"
                            placeholder="Search icons…"
                            class="h-10 w-full rounded-md border border-input bg-background pl-9 pr-3 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                            prop:value=search
                            on:input=on_search
                        />
                    </div>
                </div>

                <Show when=move || hydrated.get() && mru_visible()>
                    <div class="mb-4 flex items-center gap-3 border-b border-border pb-3">
                        <span class="flex-none font-mono text-[11px] uppercase tracking-wide text-muted-foreground">
                            "Recent"
                        </span>
                        <div class="flex flex-nowrap gap-2 overflow-x-auto">
                            {move || mru.get().iter().filter_map(|(c, name)| {
                                let glyph = c.glyph(name)?;
                                let label = name.clone();
                                let select = select_icon;
                                Some(view! {
                                    <button type="button" class="mru-cell" title=label on:click=move |_| select(glyph)>
                                        <CustomGlyphView glyph=glyph size=size_val stroke_width=sw_val stroke=stroke_val />
                                    </button>
                                }.into_any())
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                </Show>

                <div class="grid grid-cols-4 gap-2 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 xl:grid-cols-12">
                    <For
                        each=move || page_icons.get()
                        key=move |g| format!("{}:{}", collection.get().key(), g.name)
                        children=move |glyph| {
                            let kebab = glyph.name.to_string();
                            let is_animated = animated;
                            let on_click = select_icon;
                            view! {
                                <button
                                    type="button"
                                    class="flex flex-col items-center gap-1.5 rounded-lg border border-border p-2 transition-colors hover:border-ring/40 hover:bg-accent"
                                    on:click=move |_| on_click(glyph)
                                    title=kebab.clone()
                                >
                                    <Show
                                        when=move || is_animated.get()
                                        fallback=move || view! {
                                            <CustomGlyphView glyph=glyph size=size_val stroke_width=sw_val stroke=stroke_val />
                                        }
                                    >
                                        <AnimatedGlyphView glyph=glyph size=size_val stroke_width=sw_val stroke=stroke_val />
                                    </Show>
                                    <span class="w-full truncate text-center font-mono text-[9px] text-muted-foreground">
                                        {kebab.clone()}
                                    </span>
                                </button>
                            }
                        }
                    />
                </div>

                // Pagination
                <div class="mt-6 flex flex-wrap items-center justify-center gap-2">
                    <button
                        type="button"
                        class="inline-flex h-9 items-center rounded-md border border-border px-3 text-sm font-medium transition-colors hover:bg-accent disabled:pointer-events-none disabled:opacity-40"
                        disabled=move || page.get() <= 1
                        on:click=move |_| page.update(|p| *p = p.saturating_sub(1))
                    >
                        "Previous"
                    </button>
                    {move || {
                        let cur = page.get();
                        let total = total_pages.get();
                        let mut nums: Vec<usize> = Vec::new();
                        let start = cur.saturating_sub(2).max(1);
                        let end = (start + 4).min(total);
                        for n in start..=end { nums.push(n); }
                        nums.into_iter().map(|n| {
                            let is_cur = move || page.get() == n;
                            view! {
                                <button
                                    type="button"
                                    class=move || {
                                        let base = "inline-flex h-9 min-w-9 items-center justify-center rounded-md border px-3 text-sm font-medium transition-colors";
                                        if is_cur() {
                                            format!("{base} border-primary bg-primary/10 text-primary")
                                        } else {
                                            format!("{base} border-border text-muted-foreground hover:bg-accent")
                                        }
                                    }
                                    on:click=move |_| page.set(n)
                                >{n.to_string()}</button>
                            }
                        }).collect::<Vec<_>>()
                    }}
                    <span class="px-2 font-mono text-xs text-muted-foreground">
                        {move || format!("/ {}", total_pages.get())}
                    </span>
                    <button
                        type="button"
                        class="inline-flex h-9 items-center rounded-md border border-border px-3 text-sm font-medium transition-colors hover:bg-accent disabled:pointer-events-none disabled:opacity-40"
                        disabled=move || page.get() >= total_pages.get()
                        on:click=move |_| page.update(|p| *p = (*p + 1).min(total_pages.get()))
                    >
                        "Next"
                    </button>
                </div>

                // -----------------------------------------------------------
                // Detail drawer
                // -----------------------------------------------------------
                {move || selected_icon.get().map(|glyph| {
                    let name = glyph.name.to_string();
                    let svg_markup = full_svg_markup(&glyph, size_px.get(), stroke_w.get());
                    let col = collection.get();
                    let usage = if col == Collection::Lucide {
                        format!(r#"<Icon glyph=Glyph::{name} class="w-6 h-6" />"#)
                    } else {
                        format!(
                            "use montrs_icons::{{CustomIcon, Collection}};\nlet icon = Collection::{}.glyph(\"{}\").unwrap();\n<CustomIcon svg=icon.svg viewbox=icon.viewbox />",
                            col.label(), glyph.name.to_lowercase()
                        )
                    };
                    let cats: Vec<String> = if col == Collection::Lucide {
                        Glyph::by_name(glyph.name).map(|g| g.categories().map(|c| c.to_string()).collect()).unwrap_or_default()
                    } else { Vec::new() };
                    let related: Vec<CollectedGlyph> = if col == Collection::Lucide {
                        Glyph::by_name(glyph.name).map(|g| g.related(8).into_iter().map(|g| CollectedGlyph {
                            name: g.name(), svg: g.svg(), viewbox: "0 0 24 24", fill: "none", stroke: "currentColor",
                        }).collect()).unwrap_or_default()
                    } else { Vec::new() };
                    let has_related = !related.is_empty();
                    let choice = anim_choice;
                    view! {
                        <div class="icon-drawer open" role="dialog" aria-label={format!("{name} details")}>
                            <div class="p-5">
                                <div class="flex items-start justify-between">
                                    <div>
                                        <p class="font-mono text-[11px] uppercase tracking-wide text-muted-foreground">
                                            {move || collection.get().label()}
                                        </p>
                                        <h2 class="mt-1 text-lg font-semibold">{formatted_name(&name)}</h2>
                                        <p class="font-mono text-xs text-muted-foreground">{name.clone()}</p>
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

                                <div class="mt-4 flex h-40 items-center justify-center rounded-lg border border-border bg-background">
                                    <Show
                                        when=move || choice.get() != "off"
                                        fallback=move || view! {
                                            <CustomGlyphView glyph=glyph size="80" stroke_width=sw_val stroke=stroke_val />
                                        }
                                    >
                                        <AnimatedGlyphView
                                            glyph=glyph
                                            size="80"
                                            stroke_width=sw_val
                                            stroke=stroke_val
                                            profile=Signal::derive(move || match choice.get().as_str() {
                                                "draw" => Some(montrs_icons::AnimationProfile::PathDraw),
                                                "spin" => Some(montrs_icons::AnimationProfile::Spin),
                                                "pulse" => Some(montrs_icons::AnimationProfile::Pulse),
                                                "bounce" => Some(montrs_icons::AnimationProfile::Bounce),
                                                "ping" => Some(montrs_icons::AnimationProfile::Ping),
                                                "shake" => Some(montrs_icons::AnimationProfile::Shake),
                                                "nod" => Some(montrs_icons::AnimationProfile::Nod),
                                                _ => None,
                                            })
                                        />
                                    </Show>
                                </div>

                                {move || if !cats.is_empty() {
                                    view! {
                                        <div class="mt-3 flex flex-wrap gap-1.5">
                                            {cats.iter().map(|c| view! {
                                                <span class="rounded-full border border-border px-2 py-0.5 text-[11px] text-muted-foreground">{formatted_name(c)}</span>
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                } else { view! {}.into_any() }}

                                <div class="mt-4">
                                    <p class="mb-2 font-mono text-[11px] uppercase tracking-wide text-muted-foreground">"Animation"</p>
                                    <div class="flex flex-wrap gap-1.5">
                                        {anim_choices.into_iter().map(|(value, label)| {
                                            let value_for_active = value.to_string();
                                            let value_for_click = value_for_active.clone();
                                            let is_active = move || choice.get() == value_for_active;
                                            let set_choice = choice;
                                            view! {
                                                <button
                                                    type="button"
                                                    class=move || {
                                                        let base = "rounded-full border px-2.5 py-1 text-xs font-medium transition-colors";
                                                        if is_active() {
                                                            format!("{base} border-primary bg-primary/10 text-primary")
                                                        } else {
                                                            format!("{base} border-border text-muted-foreground hover:bg-accent hover:text-foreground")
                                                        }
                                                    }
                                                    on:click=move |_| set_choice.set(value_for_click.clone())
                                                >{label}</button>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>

                                <div class="mt-4 space-y-2">
                                    <div class="flex items-center gap-2 rounded-md border border-border bg-background p-2">
                                        <code class="max-h-32 flex-1 overflow-y-auto whitespace-pre-wrap text-xs">{usage.clone()}</code>
                                        <CopyButton text=usage.clone() label="Copy" />
                                    </div>
                                    <div>
                                        <p class="mb-1 font-mono text-[11px] uppercase tracking-wide text-muted-foreground">"SVG"</p>
                                        <div class="flex items-center gap-2 rounded-md border border-border bg-background p-2">
                                            <code class="max-h-20 flex-1 overflow-y-auto text-[10px] break-all">{svg_markup.clone()}</code>
                                            <CopyButton text=svg_markup.clone() label="Copy" />
                                        </div>
                                    </div>
                                </div>

                                <Show when=move || has_related>
                                    <div class="mt-5">
                                        <p class="mb-2 font-mono text-[11px] uppercase tracking-wide text-muted-foreground">"Related"</p>
                                        <div class="grid grid-cols-8 gap-1.5">
                                            {related.iter().copied().map(|g| {
                                                let select = select_icon;
                                                view! {
                                                    <button
                                                        type="button"
                                                        class="flex items-center justify-center rounded-md border border-border p-1.5 transition-colors hover:border-ring/40 hover:bg-accent"
                                                        on:click=move |_| select(g)
                                                        title=g.name
                                                    >
                                                        <CustomGlyphView glyph=g size="18" stroke_width="1.5" stroke=stroke_val />
                                                    </button>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                </Show>
                            </div>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}

/// Static render of a glyph (works for Lucide and collection tables),
/// style-aware: fill collections ignore stroke color/width.
#[component]
fn CustomGlyphView(
    glyph: CollectedGlyph,
    #[prop(into)] size: TextProp,
    #[prop(into)] stroke_width: TextProp,
    #[prop(into)] stroke: TextProp,
) -> impl IntoView {
    let size2 = size.clone();
    let is_fill = glyph.stroke == "none";
    let stroke_ok = move || {
        let c = stroke.get();
        if is_fill || c.is_empty() {
            glyph.stroke.to_string()
        } else {
            c.to_string()
        }
    };
    let sw_ok = move || {
        if is_fill {
            String::new()
        } else {
            let s = stroke_width.get();
            if s.is_empty() { "1.5".to_string() } else { s.to_string() }
        }
    };
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=move || size.get()
            height=move || size2.get()
            viewBox=move || glyph.viewbox
            fill=move || glyph.fill
            stroke=move || stroke_ok()
            stroke-width=move || sw_ok()
            stroke-linecap="round"
            stroke-linejoin="round"
            inner_html=move || glyph.svg
        />
    }
}

/// Hover-animated render of a glyph, style-aware like the static view.
#[component]
fn AnimatedGlyphView(
    glyph: CollectedGlyph,
    #[prop(into)] size: TextProp,
    #[prop(into)] stroke_width: TextProp,
    #[prop(into)] stroke: TextProp,
    #[prop(into, optional)] profile: Signal<Option<montrs_icons::AnimationProfile>>,
) -> impl IntoView {
    let is_fill = glyph.stroke == "none";
    let stroke_ok = move || {
        let c = stroke.get();
        if is_fill || c.is_empty() {
            glyph.stroke.to_string()
        } else {
            c.to_string()
        }
    };
    let sw_ok = move || {
        if is_fill {
            "1.5".to_string()
        } else {
            let s = stroke_width.get();
            if s.is_empty() { "1.5".to_string() } else { s.to_string() }
        }
    };
    view! {
        <AnimatedSvg
            svg={TextProp::from(glyph.svg)}
            viewbox={TextProp::from(glyph.viewbox)}
            fill={TextProp::from(glyph.fill)}
            stroke={TextProp::from(move || stroke_ok())}
            stroke_width={TextProp::from(move || sw_ok())}
            size=size
            profile=profile
        />
    }
}