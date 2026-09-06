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
use montrs_ui::components::switch::Switch;
use montrs_ui::prelude::*;

const CHUNK_SIZE: usize = 240;

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

fn full_svg_markup(glyph: Glyph, size: u32, stroke_w: f64) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" \
         height=\"{size}\" viewBox=\"0 0 24 24\" fill=\"none\" \
         stroke=\"currentColor\" stroke-width=\"{stroke_w}\" \
         stroke-linecap=\"round\" stroke-linejoin=\"round\">{}</svg>",
        glyph.svg()
    )
}

// ---------------------------------------------------------------------------
// MRU (localStorage, client-only)
// ---------------------------------------------------------------------------

fn load_mru() -> Vec<Glyph> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
            && let Ok(Some(raw)) = storage.get_item("montrs-icons-mru")
            && let Ok(names) = serde_json::from_str::<Vec<String>>(&raw)
        {
            return names
                .iter()
                .filter_map(|n| Glyph::by_name(n))
                .collect::<Vec<_>>();
        }
    }
    Vec::new()
}

#[allow(unused_variables)]
fn save_mru(items: &[Glyph]) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let names: Vec<String> =
                items.iter().map(|g| g.name().to_string()).collect();
            if let Ok(json) = serde_json::to_string(&names)
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
        query.get().get("anim").is_some_and(|v| v == "1"),
    );

    let hydrated = RwSignal::new(false);
    let mru = RwSignal::new(Vec::<Glyph>::new());
    let visible_count = RwSignal::new(CHUNK_SIZE);
    let selected_icon = RwSignal::new(None::<Glyph>);
    let anim_choice = RwSignal::new("auto".to_string());

    // Set MRU + hydration flag after mount so SSR and hydration stay in sync.
    Effect::new(move |_| {
        if !hydrated.get() {
            hydrated.set(true);
            mru.set(load_mru());
        }
    });

    // Reset the visible chunk whenever the filter changes.
    Effect::new(move |_| {
        search.get();
        category.get();
        visible_count.set(CHUNK_SIZE);
    });

    let filtered = Memo::new(move |_| {
        let s = search.get();
        let cat = category.get();
        let mut found = if s.is_empty() {
            Glyph::find("")
        } else {
            Glyph::find(&s)
        };
        if !cat.is_empty() {
            found.retain(|g| g.categories().any(|c| c.eq_ignore_ascii_case(&cat)));
        }
        found
    });

    let filtered_limited = Memo::new(move |_| {
        let mut v = filtered.get();
        v.truncate(visible_count.get());
        v
    });

    let categories = Glyph::all_categories();

    // Infinite scroll: observe the sentinel and extend the chunk.
    Effect::new(move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                let cb = wasm_bindgen::prelude::Closure::wrap(Box::new(
                    move |entries: js_sys::Array| {
                        for i in 0..entries.length() {
                            if let Some(entry) = entries
                                .get(i)
                                .dyn_ref::<web_sys::IntersectionObserverEntry>()
                                && entry.is_intersecting()
                            {
                                visible_count.update(|c| *c += CHUNK_SIZE);
                            }
                        }
                    },
                ) as Box<dyn FnMut(js_sys::Array)>);
                if let Ok(observer) = web_sys::IntersectionObserver::new(
                    cb.as_ref().unchecked_ref(),
                ) {
                    if let Some(el) =
                        document.get_element_by_id("icons-sentinel")
                    {
                        observer.observe(&el);
                    }
                }
                cb.forget();
            }
        }
    });

    let select_icon = move |glyph: Glyph| {
        selected_icon.set(Some(glyph));
        anim_choice.set("auto".to_string());
        mru.update(|v| {
            v.retain(|g| *g != glyph);
            v.insert(0, glyph);
            v.truncate(8);
            save_mru(v);
        });
    };

    let clear_filters = move |_: leptos::ev::MouseEvent| {
        search.set(String::new());
        category.set(String::new());
    };

    // Per-control navigation that mirrors state to the URL (replace, so
    // the history doesn't fill with slider ticks).
    let sync_url = {
        let nav = navigate.clone();
        move || {
            let mut q = format!("/ui/icons?size={}&sw={}", size_px.get(), stroke_w.get());
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
            if animated.get() {
                q.push_str("&anim=1");
            }
            nav(&q, NavigateOptions { replace: true, ..Default::default() });
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
                size_px.set(v);
            }
            sync();
        }
    };
    let on_stroke_w = {
        let sync = sync_url.clone();
        move |e: leptos::ev::Event| {
            if let Ok(v) = event_target_value(&e).parse::<f64>() {
                stroke_w.set(v);
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

    let size_text = move || format!("{}px", size_px.get());
    let sw_text = move || format!("{:.2}px", stroke_w.get());
    let stroke_val = Signal::derive(move || {
        let c = color.get();
        if c.is_empty() {
            "currentColor".to_string()
        } else {
            c
        }
    });
    let size_val = Signal::derive(move || size_px.get().to_string());
    let sw_val = Signal::derive(move || format!("{:.2}", stroke_w.get()));
    let mru_visible = move || search.get().is_empty() && category.get().is_empty();

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
                                    {move || format!("{} icons", Glyph::count())}
                                </p>
                            </div>
                        </div>
                    </div>

                    <div class="icons-sidebar-section">
                        <p class="icons-sidebar-heading">"Customize"</p>
                        <div class="space-y-4">
                            <label class="block">
                                <span class="flex justify-between text-xs text-muted-foreground">
                                    "Size"
                                    <span class="font-mono text-foreground">{size_text}</span>
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
                                <span class="flex justify-between text-xs text-muted-foreground">
                                    "Stroke width"
                                    <span class="font-mono text-foreground">{sw_text}</span>
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
                            <label class="flex items-center justify-between text-xs text-muted-foreground">
                                "Stroke color"
                                <input
                                    type="color"
                                    class="h-7 w-9 cursor-pointer rounded border border-border bg-transparent"
                                    prop:value=move || color.get()
                                    on:input=on_color
                                />
                            </label>
                            <label class="flex items-center justify-between text-xs text-muted-foreground">
                                "Animated"
                                <Switch checked=animated />
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
                            {move || format!("{} shown · hover to play", filtered_limited.get().len())}
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

                // MRU strip (client-only, appears after hydration)
                <Show when=move || hydrated.get() && mru_visible()>
                    <div class="mb-4 flex items-center gap-3 border-b border-border pb-3">
                        <span class="flex-none font-mono text-[11px] uppercase tracking-wide text-muted-foreground">
                            "Recent"
                        </span>
                        <div class="flex flex-nowrap gap-2 overflow-x-auto">
                            {move || mru.get().iter().map(|g| {
                                let glyph = *g;
                                let select = select_icon;
                                view! {
                                    <button
                                        type="button"
                                        class="mru-cell"
                                        title=g.kebab_name()
                                        on:click=move |_| select(glyph)
                                    >
                                        <Icon
                                            glyph=Signal::from(glyph)
                                            size=size_val
                                            stroke_width=sw_val
                                            stroke=stroke_val
                                        />
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                </Show>

                <div class="grid grid-cols-4 gap-2 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 xl:grid-cols-12">
                    <For
                        each=move || filtered_limited.get()
                        key=|g| *g
                        children=move |glyph| {
                            let kebab = glyph.kebab_name();
                            let name = glyph.name().to_string();
                            let is_animated = animated;
                            let on_click = select_icon;
                            view! {
                                <button
                                    type="button"
                                    class="flex flex-col items-center gap-1.5 rounded-lg border border-border p-2 transition-colors hover:border-ring/40 hover:bg-accent"
                                    on:click=move |_| on_click(glyph)
                                    title=name.clone()
                                >
                                    <Show
                                        when=move || is_animated.get()
                                        fallback=move || view! {
                                            <Icon
                                                glyph=Signal::from(glyph)
                                                size=size_val
                                                stroke_width=sw_val
                                                stroke=stroke_val
                                            />
                                        }
                                    >
                                        <AnimatedIcon
                                            glyph=Signal::from(glyph)
                                            size=size_val
                                            stroke_width=sw_val
                                            stroke=stroke_val
                                        />
                                    </Show>
                                    <span class="w-full truncate text-center font-mono text-[9px] text-muted-foreground">
                                        {kebab}
                                    </span>
                                </button>
                            }
                        }
                    />
                </div>

                <div id="icons-sentinel" class="h-10"></div>

                // -----------------------------------------------------------
                // Detail drawer
                // -----------------------------------------------------------
                {move || selected_icon.get().map(|glyph| {
                    let name = glyph.name().to_string();
                    let kebab = glyph.kebab_name().to_string();
                    let svg_markup = full_svg_markup(glyph, size_px.get(), stroke_w.get());
                    let usage = if animated.get() {
                        format!(r#"<AnimatedIcon glyph=Glyph::{name} class="w-6 h-6" />"#)
                    } else {
                        format!(r#"<Icon glyph=Glyph::{name} class="w-6 h-6" />"#)
                    };
                    let cats: Vec<&str> = glyph.categories().collect();
                    let tags = glyph.tags().collect::<Vec<_>>().join(" • ");
                    let related = glyph.related(8);
                    let has_related = !related.is_empty();
                    let choice = anim_choice;
                    view! {
                        <div class="icon-drawer open" role="dialog" aria-label={format!("{name} details")}>
                            <div class="p-5">
                                <div class="flex items-start justify-between">
                                    <div>
                                        <p class="font-mono text-[11px] uppercase tracking-wide text-muted-foreground">"Icon"</p>
                                        <h2 class="mt-1 text-lg font-semibold">{formatted_name(&kebab)}</h2>
                                        <p class="font-mono text-xs text-muted-foreground">{kebab.clone()}</p>
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
                                            <Icon glyph=Signal::from(glyph) size="80" stroke_width="2" />
                                        }
                                    >
                                        <AnimatedIcon
                                            glyph=Signal::from(glyph)
                                            size="80"
                                            stroke_width="2"
                                            profile=Signal::derive(move || match choice.get().as_str() {
                                                "draw" => Some(AnimationProfile::PathDraw),
                                                "spin" => Some(AnimationProfile::Spin),
                                                "pulse" => Some(AnimationProfile::Pulse),
                                                "bounce" => Some(AnimationProfile::Bounce),
                                                "ping" => Some(AnimationProfile::Ping),
                                                "shake" => Some(AnimationProfile::Shake),
                                                "nod" => Some(AnimationProfile::Nod),
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

                                {move || if !tags.is_empty() {
                                    view! {
                                        <p class="mt-2 text-xs text-muted-foreground">{tags.clone()}</p>
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
                                        <code class="flex-1 truncate text-xs">{usage.clone()}</code>
                                        <CopyButton text=usage.clone() label="Copy" />
                                    </div>
                                    <div>
                                        <p class="mb-1 font-mono text-[11px] uppercase tracking-wide text-muted-foreground">"SVG"</p>
                                        <div class="flex items-center gap-2 rounded-md border border-border bg-background p-2">
                                            <code class="max-h-20 flex-1 overflow-y-auto text-[10px] break-all">{svg_markup.clone()}</code>
                                            <CopyButton text=svg_markup.clone() label="Copy" />
                                        </div>
                                    </div>
                                    <div class="flex items-center gap-2 rounded-md border border-border bg-background p-2">
                                        <code class="flex-1 truncate text-xs">{format!("use montrs_icons::Glyph;")}</code>
                                        <CopyButton text="use montrs_icons::Glyph;".to_string() label="Copy" />
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
                                                        title=g.kebab_name()
                                                    >
                                                        <Icon glyph=Signal::from(g) size="18" />
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