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
use montrs_icons::*;
use montrs_ui::prelude::*;

#[component]
pub fn Ui() -> impl IntoView {
    let sections = [
        (
            "/ui/components",
            Glyph::Blocks,
            "Components",
            "91 shadcn-inspired components built on Tailwind CSS. Buttons, \
             cards, inputs, tabs, and more — copy the source, own every pixel.",
            "h-40",
        ),
        (
            "/ui/blocks",
            Glyph::LayoutTemplate,
            "Blocks",
            "Pre-built UI sections from real MontRS Plates: FAQs, footers, \
             headers, login screens, sidenavs, integrations.",
            "h-40",
        ),
        (
            "/ui/icons",
            Glyph::Palette,
            "Icons",
            "1,700+ Lucide icons as Leptos components, with search, sizing, \
             stroke, and fill controls. Toggle animated hover effects.",
            "h-40",
        ),
        (
            "/ui/motion",
            Glyph::Activity,
            "Motion",
            "Spring physics, tween easings, gestures, and SVG path animation \
             from the montrs-motion package.",
            "h-40",
        ),
    ];

    view! {
        <div class="page-container py-12">
            <div class="mb-10">
                <h1 class="text-3xl font-bold tracking-tight">"UI"</h1>
                <p class="mt-2 max-w-2xl text-muted-foreground">
                    "The full montrs-ui experience: components, blocks, icons, and motion."
                </p>
            </div>

            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                {sections.into_iter().map(|(href, icon, title, desc, _h)| view! {
                    <a href=href class="showcase-card reveal flex flex-col justify-between p-8">
                        <div>
                            <Icon glyph=icon class="h-8 w-8 text-primary" />
                            <h2 class="mt-4 text-2xl font-semibold">{title}</h2>
                            <p class="mt-2 text-sm leading-6 text-muted-foreground">{desc}</p>
                        </div>
                        <span class="mt-6 inline-flex items-center gap-1.5 text-sm font-medium text-primary">
                            "Explore"
                            <Icon glyph=Glyph::ArrowRight class="h-4 w-4" />
                        </span>
                    </a>
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
