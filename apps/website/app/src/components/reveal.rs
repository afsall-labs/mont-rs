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

//! Scroll-triggered reveal: elements with `.reveal` fade/slide in when they
//! enter the viewport. No-op during SSR (no DOM), so hydration stays in sync.

use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn RevealOnScroll() -> impl IntoView {
    #[allow(unused_variables)]
    let first_run = RwSignal::new(true);

    // Re-run on every route change: new pages mount fresh `.reveal`
    // elements, and without re-observing them they'd stay `opacity:0` until a
    // manual refresh (the "page looks empty until I refresh" bug).
    Effect::new(move |_| {
        let location = use_location();
        let _path = location.pathname.get();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;

            let Some(window) = web_sys::window() else {
                return;
            };
            let Some(document) = window.document() else {
                return;
            };

            // After the very first paint, scroll to the top on navigation so
            // the next page is seen from its header (matches SPA behavior).
            if first_run.get_untracked() {
                first_run.set(false);
            } else {
                let _ = window.scroll_to_with_x_and_y(0.0, 0.0);
            }

            let callback = {
                let document = document.clone();
                wasm_bindgen::prelude::Closure::wrap(Box::new(
                    move |entries: js_sys::Array| {
                        for i in 0..entries.length() {
                            if let Some(entry) = entries
                                .get(i)
                                .dyn_ref::<web_sys::IntersectionObserverEntry>()
                            {
                                if entry.is_intersecting() {
                                    if let Some(el) = entry
                                        .target()
                                        .dyn_ref::<web_sys::Element>()
                                    {
                                        let _ = el
                                            .class_list()
                                            .add_1("is-visible");
                                        let _ = &document;
                                    }
                                }
                            }
                        }
                    },
                )
                    as Box<dyn FnMut(js_sys::Array)>)
            };

            let observer = web_sys::IntersectionObserver::new(
                callback.as_ref().unchecked_ref(),
            );
            let Ok(observer) = observer else {
                reveal_all(&document);
                return;
            };

            if let Ok(elements) = document.query_selector_all(".reveal") {
                for i in 0..elements.length() {
                    if let Some(el) = elements.item(i).and_then(|node| {
                        node.dyn_into::<web_sys::Element>().ok()
                    }) {
                        observer.observe(&el);
                    }
                }
            }

            callback.forget();
        }
    });

    view! {
        // A real, hidden node: a component that renders truly *nothing*
        // between siblings desynchronizes the SSR marker walk from hydration
        // (SSR emits no marker for an empty view, but hydration expects one).
        <span class="hidden" aria-hidden="true"></span>
    }
}

#[cfg(target_arch = "wasm32")]
fn reveal_all(document: &web_sys::Document) {
    use wasm_bindgen::JsCast;
    if let Ok(elements) = document.query_selector_all(".reveal") {
        for i in 0..elements.length() {
            if let Some(el) = elements
                .item(i)
                .and_then(|node| node.dyn_into::<web_sys::Element>().ok())
            {
                let _ = el.class_list().add_1("is-visible");
            }
        }
    }
}
