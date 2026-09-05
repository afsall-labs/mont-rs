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
use montrs_motion::*;
use montrs_ui::{components::slider::Slider, prelude::*};

#[component]
pub fn Motion() -> impl IntoView {
    view! {
        <div class="page-container py-12">
            <div class="mb-10">
                <h1 class="text-3xl font-bold tracking-tight">"Motion & Animation"</h1>
                <p class="mt-2 max-w-2xl text-muted-foreground">
                    "Interactive demos of spring physics, tween easing, and SVG path
                    animation — all driven by the montrs-motion FrameLoop."
                </p>
            </div>

            <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
                <SpringDemo />
                <TweenDemo />
            </div>

            <div class="mt-6">
                <PathDemo />
            </div>
        </div>
    }
}

#[component]
fn SpringDemo() -> impl IntoView {
    let stiffness = RwSignal::new(100.0);
    let damping = RwSignal::new(10.0);
    let mass = RwSignal::new(1.0);
    let value = RwSignal::new(0.0);
    let playing = RwSignal::new(false);

    let play_spring = move |_| {
        if playing.get() {
            return;
        }
        playing.set(true);
        value.set(0.0);
        let spring = Spring::new(stiffness.get(), damping.get(), mass.get())
            .with_range(0.0, 1.0);
        let start = FrameLoop::now();
        FrameLoop::on_frame(move || {
            let elapsed = FrameLoop::now() - start;
            value.set(spring.solve(elapsed));
            if elapsed > 2.0 {
                playing.set(false);
                false
            } else {
                true
            }
        });
    };

    view! {
        <div class="showcase-card p-6">
            <div class="flex items-center gap-2">
                <Icon glyph=Glyph::Activity class="h-5 w-5 text-primary" />
                <h2 class="text-xl font-semibold">"Spring Physics"</h2>
            </div>
            <p class="mt-1 text-sm text-muted-foreground">
                "Tune the spring and watch the mass settle."
            </p>

            <div class="mt-6 space-y-5">
                <div>
                    <label class="flex justify-between text-sm font-medium">
                        "Stiffness"
                        <span class="font-mono text-muted-foreground">{move || format!("{:.0}", stiffness.get())}</span>
                    </label>
                    <Slider min=10.0 max=500.0 step=1.0 value=stiffness class="mt-2" />
                </div>
                <div>
                    <label class="flex justify-between text-sm font-medium">
                        "Damping"
                        <span class="font-mono text-muted-foreground">{move || format!("{:.1}", damping.get())}</span>
                    </label>
                    <Slider min=1.0 max=50.0 step=0.5 value=damping class="mt-2" />
                </div>
                <div>
                    <label class="flex justify-between text-sm font-medium">
                        "Mass"
                        <span class="font-mono text-muted-foreground">{move || format!("{:.1}", mass.get())}</span>
                    </label>
                    <Slider min=0.1 max=10.0 step=0.1 value=mass class="mt-2" />
                </div>

                <button
                    type="button"
                    class="inline-flex h-10 items-center gap-2 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground shadow-sm transition-colors hover:bg-primary/90 disabled:opacity-50"
                    on:click=play_spring
                    disabled=move || playing.get()
                >
                    <Icon glyph=Glyph::Play class="h-4 w-4" />
                    {move || if playing.get() { "Playing…" } else { "Play Spring" }}
                </button>
            </div>

            <div class="mt-6 flex items-end gap-1 rounded-md border border-border bg-background p-4" style="height: 140px;">
                {move || {
                    let pct = (value.get() * 100.0).min(100.0);
                    view! {
                        <div class="w-full rounded-t bg-primary" style=format!("height: {}%;", pct)></div>
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn TweenDemo() -> impl IntoView {
    let easing_idx = RwSignal::new(0usize);
    let progress = RwSignal::new(0.0);
    let playing = RwSignal::new(false);

    let easings = [
        ("Linear", Easing::Linear),
        ("Ease", Easing::Ease),
        ("EaseIn", Easing::EaseIn),
        ("EaseOut", Easing::EaseOut),
        ("EaseInOut", Easing::EaseInOut),
        ("QuadIn", Easing::QuadIn),
        ("QuadOut", Easing::QuadOut),
        ("CubicIn", Easing::CubicIn),
        ("CubicOut", Easing::CubicOut),
        ("SineIn", Easing::SineIn),
        ("SineOut", Easing::SineOut),
        ("BackOut", Easing::BackOut),
        ("ElasticOut", Easing::ElasticOut),
        ("BounceOut", Easing::BounceOut),
    ];

    let play_tween = move |_| {
        if playing.get() {
            return;
        }
        playing.set(true);
        progress.set(0.0);
        let (_, easing) = easings[easing_idx.get()];
        let tween = Tween::new(0.0, 1.0, 1.0).with_easing(easing);
        let start = FrameLoop::now();
        FrameLoop::on_frame(move || {
            let elapsed = FrameLoop::now() - start;
            progress.set(tween.sample(elapsed));
            if elapsed > 1.0 {
                playing.set(false);
                false
            } else {
                true
            }
        });
    };

    view! {
        <div class="showcase-card p-6">
            <div class="flex items-center gap-2">
                <Icon glyph=Glyph::ChartSpline class="h-5 w-5 text-primary" />
                <h2 class="text-xl font-semibold">"Tween Easing Visualizer"</h2>
            </div>
            <p class="mt-1 text-sm text-muted-foreground">
                "Pick an easing function and watch the curve."
            </p>

            <div class="mt-6 space-y-5">
                <div>
                    <label class="mb-2 block text-sm font-medium">"Easing"</label>
                    <select
                        class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
                        prop:value=move || easing_idx.get().to_string()
                        on:change=move |e| {
                            let val = event_target_value(&e);
                            if let Ok(idx) = val.parse::<usize>() {
                                easing_idx.set(idx);
                            }
                        }
                    >
                        {easings.iter().enumerate().map(|(i, (name, _))| {
                            view! { <option value=i.to_string()>{*name}</option> }
                        }).collect::<Vec<_>>()}
                    </select>
                </div>

                <button
                    type="button"
                    class="inline-flex h-10 items-center gap-2 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground shadow-sm transition-colors hover:bg-primary/90 disabled:opacity-50"
                    on:click=play_tween
                    disabled=move || playing.get()
                >
                    <Icon glyph=Glyph::Play class="h-4 w-4" />
                    {move || if playing.get() { "Playing…" } else { "Play Tween" }}
                </button>
            </div>

            <div class="mt-6">
                <svg viewBox="0 0 200 120" class="h-32 w-full rounded-md border border-border bg-background">
                    <line x1="0" y1="120" x2="200" y2="120" stroke="currentColor" stroke-width="1" opacity="0.25" />
                    <line x1="0" y1="0" x2="0" y2="120" stroke="currentColor" stroke-width="1" opacity="0.25" />
                    {move || {
                        let pts: Vec<String> = (0..=50).map(|i| {
                            let t = i as f64 / 50.0;
                            let (_, easing) = easings[easing_idx.get()];
                            let y = 1.0 - easing.apply(t);
                            let px = 4.0 + t * 192.0;
                            let py = 4.0 + y * 112.0;
                            format!("{:.1},{:.1}", px, py)
                        }).collect();
                        let d = format!("M{}", pts.join(" L"));
                        view! {
                            <path d=d stroke="hsl(var(--primary))" stroke-width="2" fill="none" />
                        }
                    }}
                    {move || {
                        let p = progress.get();
                        let (_, easing) = easings[easing_idx.get()];
                        let y = 1.0 - easing.apply(p);
                        let cx = 4.0 + p * 192.0;
                        let cy = 4.0 + y * 112.0;
                        view! {
                            <circle cx=cx.to_string() cy=cy.to_string() r="5" fill="hsl(var(--primary))" />
                        }
                    }}
                </svg>
            </div>
        </div>
    }
}

#[component]
fn PathDemo() -> impl IntoView {
    let progress = RwSignal::new(0.0);
    let playing = RwSignal::new(false);

    let play_path = move |_| {
        if playing.get() {
            return;
        }
        playing.set(true);
        progress.set(0.0);
        let start = FrameLoop::now();
        FrameLoop::on_frame(move || {
            let elapsed = FrameLoop::now() - start;
            progress.set((elapsed / 2.0).min(1.0));
            if elapsed > 2.0 {
                playing.set(false);
                false
            } else {
                true
            }
        });
    };

    view! {
        <div class="showcase-card p-6">
            <div class="flex flex-wrap items-center justify-between gap-4">
                <div>
                    <div class="flex items-center gap-2">
                        <Icon glyph=Glyph::PenLine class="h-5 w-5 text-primary" />
                        <h2 class="text-xl font-semibold">"SVG Path Animation"</h2>
                    </div>
                    <p class="mt-1 text-sm text-muted-foreground">
                        "Stroke-dasharray / stroke-dashoffset path drawing."
                    </p>
                </div>
                <div class="flex items-center gap-4">
                    <button
                        type="button"
                        class="inline-flex h-10 items-center gap-2 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground shadow-sm transition-colors hover:bg-primary/90 disabled:opacity-50"
                        on:click=play_path
                        disabled=move || playing.get()
                    >
                        <Icon glyph=Glyph::Play class="h-4 w-4" />
                        {move || if playing.get() { "Drawing…" } else { "Animate Path" }}
                    </button>
                    <span class="font-mono text-sm text-muted-foreground">
                        {move || format!("{:.0}%", progress.get() * 100.0)}
                    </span>
                </div>
            </div>

            <div class="mt-6 flex justify-center rounded-md border border-border bg-background p-6">
                <svg viewBox="0 0 200 120" class="h-40 w-64">
                    {move || {
                        let length = 280.0;
                        let offset = length * (1.0 - progress.get());
                        view! {
                            <path
                                d="M20 100 Q50 10 100 60 T180 40"
                                stroke="hsl(var(--primary))"
                                stroke-width="3"
                                fill="none"
                                stroke-linecap="round"
                                stroke-dasharray=format!("{} {}", length, length)
                                stroke-dashoffset=offset.to_string()
                            />
                        }
                    }}
                </svg>
            </div>
        </div>
    }
}
