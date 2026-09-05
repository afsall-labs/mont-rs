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

//! Small client-only helpers: clipboard copy and copy-button component.

use leptos::prelude::*;

/// Write `text` to the clipboard. Returns `true` when the write was accepted.
#[cfg(target_arch = "wasm32")]
pub fn copy_text(text: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let clipboard = window.navigator().clipboard();

    let promise = clipboard.write_text(text);
    // Attach a no-op catch so a rejected promise (e.g. permissions) doesn't
    // produce an unhandledrejection in the console.
    let mut closure = wasm_bindgen::prelude::Closure::wrap(Box::new(
        |_: wasm_bindgen::JsValue| {},
    )
        as Box<dyn FnMut(wasm_bindgen::JsValue)>);
    let _ = promise.catch(&mut closure);
    true
}

#[cfg(not(target_arch = "wasm32"))]
pub fn copy_text(_text: &str) -> bool {
    false
}

/// A button that copies `text` to the clipboard and flashes "Copied" for 1.5s.
#[component]
pub fn CopyButton(
    #[prop(into)] text: String,
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] label: String,
) -> impl IntoView {
    let copied = RwSignal::new(false);
    let label = if label.is_empty() {
        "Copy".to_string()
    } else {
        label
    };

    let on_click = move |_| {
        copy_text(&text);
        copied.set(true);
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;
            let copied2 = copied;
            let cb = Closure::wrap(
                Box::new(move || copied2.set(false)) as Box<dyn FnMut()>
            );
            if let Some(window) = web_sys::window() {
                let _ = window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        cb.as_ref().unchecked_ref(),
                        1500,
                    );
            }
            cb.forget();
        }
    };

    view! {
        <button
            type="button"
            class=move || {
                let state = if copied.get() {
                    " border-transparent bg-primary/15 text-primary"
                } else {
                    ""
                };
                let user = if class.is_empty() { "" } else { &class };
                format!("copy-btn{state} {user}")
            }
            on:click=on_click
        >
            {move || if copied.get() { "Copied".to_string() } else { label.clone() }}
        </button>
    }
}
