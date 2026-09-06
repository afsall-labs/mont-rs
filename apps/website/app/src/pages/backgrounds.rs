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
use montrs_ui::prelude::*;

const PATTERNS: &str = r#"@layer components {
  .bg-grid {
    background-image:
      linear-gradient(hsl(var(--foreground) / 0.05) 1px, transparent 1px),
      linear-gradient(90deg, hsl(var(--foreground) / 0.05) 1px, transparent 1px);
    background-size: 40px 40px;
  }
  .bg-dots {
    background-image: radial-gradient(hsl(var(--foreground) / 0.09) 1px, transparent 1px);
    background-size: 18px 18px;
  }
  .bg-diagonal {
    background-image: repeating-linear-gradient(
      -45deg,
      hsl(var(--foreground) / 0.06) 0,
      hsl(var(--foreground) / 0.06) 1px,
      transparent 1px,
      transparent 12px
    );
  }
}"#;

#[component]
pub fn Backgrounds() -> impl IntoView {
    let patterns = [
        ("Grid", "bg-grid", "A subtle 40px grid — great for hero sections."),
        ("Grid fade", "bg-grid-fade", "Grid with a soft radial fade-out."),
        ("Dots", "bg-dots", "Tight 18px dot matrix."),
        ("Dots large", "bg-dots-lg", "Looser 32px dots for spacious surfaces."),
        ("Diagonal", "bg-diagonal", "Thin 45° diagonal lines."),
        ("Diagonal wide", "bg-diagonal-lines", "Wider spaced diagonal streaks."),
        ("Boxes", "bg-boxes", "A larger 80px checker grid."),
        ("Radial glow", "bg-radial-glow", "A soft primary-colored glow from the top."),
    ];

    view! {
        <div class="page-container py-12">
            <div class="mb-10">
                <h1 class="text-3xl font-bold tracking-tight">"Backgrounds"</h1>
                <p class="mt-2 max-w-2xl text-muted-foreground">
                    "Ready-made CSS backgrounds — copy the class (or the full rule)
                    and drop it into your MontRS app. Pure CSS, no assets."
                </p>
            </div>

            <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                {patterns.into_iter().map(|(name, cls, desc)| {
                    let css = format!(".{cls} /* {desc} */");
                    view! {
                        <div class="showcase-card overflow-hidden">
                            <div class={format!("h-40 w-full border-b border-border {cls}")}></div>
                            <div class="flex items-center justify-between gap-2 p-4">
                                <div>
                                    <p class="font-mono text-sm font-medium">{name}</p>
                                    <p class="mt-0.5 text-xs text-muted-foreground">{desc}</p>
                                </div>
                                <CopyButton text=css label="Copy" />
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="code-window mt-10 max-w-2xl">
                <div class="code-window-bar">
                    <span class="traffic-light traffic-light-red"></span>
                    <span class="traffic-light traffic-light-yellow"></span>
                    <span class="traffic-light traffic-light-green"></span>
                    <span class="code-window-tab">"backgrounds.css"</span>
                </div>
                <pre class="code-window-body text-left" inner_html=move || crate::highlight::escape_html(PATTERNS)></pre>
            </div>
        </div>
    }
}