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

use crate::components::ThemeCustomizer;
use leptos::prelude::*;
use montrs_ui::components::badge::{Badge, BadgeVariant};
use montrs_ui::components::button::{Button, ButtonVariant};
use montrs_ui::components::card::{Card, CardContent, CardHeader, CardTitle};
use montrs_ui::components::input::Input;
use montrs_ui::components::switch::Switch;
use montrs_ui::prelude::*;

#[component]
pub fn Themes() -> impl IntoView {
    let switch_on = RwSignal::new(true);
    view! {
        <div class="page-container py-12">
            <div class="mb-10">
                <h1 class="text-3xl font-bold tracking-tight">"Themes"</h1>
                <p class="mt-2 max-w-2xl text-muted-foreground">
                    "Pick a color. Make it yours. The customizer applies your
                    choices instantly across the whole site and emits the CSS
                    variables so you can paste them into your own app."
                </p>
            </div>

            <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
                <Card>
                    <CardHeader>
                        <CardTitle>"Theme Editor"</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <ThemeCustomizer />
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader>
                        <CardTitle>"Live preview"</CardTitle>
                    </CardHeader>
                    <CardContent class="space-y-4">
                        <div class="flex flex-wrap items-center gap-3">
                            <Button>"Primary"</Button>
                            <Button variant=ButtonVariant::Secondary>"Secondary"</Button>
                            <Button variant=ButtonVariant::Outline>"Outline"</Button>
                            <Button variant=ButtonVariant::Ghost>"Ghost"</Button>
                            <Button variant=ButtonVariant::Destructive>"Danger"</Button>
                        </div>
                        <div class="flex flex-wrap items-center gap-3">
                            <Badge>"New"</Badge>
                            <Badge variant=BadgeVariant::Secondary>"Secondary"</Badge>
                            <Badge variant=BadgeVariant::Outline>"Outline"</Badge>
                            <StatusDot />
                            <Switch checked=switch_on />
                            <span class="text-sm text-muted-foreground">
                                {move || if switch_on.get() { "Enabled" } else { "Disabled" }}
                            </span>
                        </div>
                        <Input placeholder="Type something…" />
                        <div class="grid grid-cols-3 gap-2 text-center font-mono text-xs text-muted-foreground">
                            <div class="rounded-md border border-border p-3">"one"</div>
                            <div class="rounded-md border border-border p-3">"two"</div>
                            <div class="rounded-md border border-border p-3">"three"</div>
                        </div>
                    </CardContent>
                </Card>
            </div>
        </div>
    }
}

#[component]
fn StatusDot() -> impl IntoView {
    view! {
        <span class="inline-flex items-center gap-1.5 text-xs text-muted-foreground">
            <span class="h-2 w-2 rounded-full bg-green-500"></span>
            "online"
        </span>
    }
}