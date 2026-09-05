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

use crate::{copy::CopyButton, highlight::highlight_rust};
use leptos::prelude::*;
use montrs_icons::*;
use montrs_ui::components::{
    accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger},
    badge::{Badge, BadgeSize, BadgeVariant},
    button::{Button, ButtonSize, ButtonVariant},
    card::{Card, CardContent, CardDescription, CardHeader, CardTitle},
    input::Input,
    switch::Switch,
    tabs::{Tabs, TabsContent, TabsList, TabsTrigger},
};

const BUTTON_SNIPPET: &str = r#"use montrs_ui::components::button::{Button, ButtonVariant};

<Button>Default</Button>
<Button variant=ButtonVariant::Outline>Outline</Button>
<Button variant=ButtonVariant::Destructive>Delete</Button>
<Button size=ButtonSize::Sm>Small</Button>
<Button size=ButtonSize::Icon>
    <Icon glyph=Glyph::Search class="h-4 w-4" />
</Button>"#;

const BADGE_SNIPPET: &str = r#"use montrs_ui::components::badge::{Badge, BadgeVariant};

<Badge>Default</Badge>
<Badge variant=BadgeVariant::Secondary>Secondary</Badge>
<Badge variant=BadgeVariant::Outline>Outline</Badge>
<Badge variant=BadgeVariant::Destructive>Destructive</Badge>"#;

const CARD_SNIPPET: &str = r#"use montrs_ui::components::card::*;

<Card>
    <CardHeader>
        <CardTitle>"Deployments"</CardTitle>
        <CardDescription>"Manage your live services"</CardDescription>
    </CardHeader>
    <CardContent>
        "42 services running · 3 pending"
    </CardContent>
</Card>"#;

const INPUT_SNIPPET: &str = r#"use montrs_ui::components::input::Input;

let value = RwSignal::new(String::new());

<Input placeholder="Search packages…" value=value />"#;

const SWITCH_SNIPPET: &str = r#"use montrs_ui::components::switch::Switch;

let enabled = RwSignal::new(true);

<Switch checked=enabled />
<span>{move || if enabled.get() { "On" } else { "Off" }}</span>"#;

const TABS_SNIPPET: &str = r#"use montrs_ui::components::tabs::*;

<Tabs default_value="preview">
    <TabsList>
        <TabsTrigger value="preview">"Preview"</TabsTrigger>
        <TabsTrigger value="code">"Code"</TabsTrigger>
    </TabsList>
    <TabsContent value="preview">"Live preview"</TabsContent>
    <TabsContent value="code">"Source code"</TabsContent>
</Tabs>"#;

const ACCORDION_SNIPPET: &str = r#"use montrs_ui::components::accordion::*;

<Accordion>
    <AccordionItem value="what">
        <AccordionTrigger>"What is a Plate?"</AccordionTrigger>
        <AccordionContent>"A feature module with explicit trait boundaries."</AccordionContent>
    </AccordionItem>
    <AccordionItem value="why">
        <AccordionTrigger>"Why deterministic?"</AccordionTrigger>
        <AccordionContent>"Same input, same output — everywhere."</AccordionContent>
    </AccordionItem>
</Accordion>"#;

const SECTIONS: &[(&str, &str)] = &[
    ("button", "Button"),
    ("badge", "Badge"),
    ("card", "Card"),
    ("input", "Input"),
    ("switch", "Switch"),
    ("tabs", "Tabs"),
    ("accordion", "Accordion"),
];

/// Smooth-scroll to an element id without touching the URL (anchor links
/// would be swallowed by the router and bounce you to the root).
fn scroll_to(id: &'static str) -> impl Fn(leptos::ev::MouseEvent) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = id;
    move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            if let Some(doc) = web_sys::window().and_then(|w| w.document())
                && let Some(el) = doc.get_element_by_id(id)
            {
                if let Some(html) = el.dyn_ref::<web_sys::HtmlElement>() {
                    let _ = html.scroll_into_view();
                }
            }
        }
    }
}

#[component]
pub fn Components() -> impl IntoView {
    view! {
        <div class="page-container py-12">
            <div class="mb-10">
                <h1 class="text-3xl font-bold tracking-tight">"Components"</h1>
                <p class="mt-2 max-w-2xl text-muted-foreground">
                    "91 shadcn-inspired components built on montrs-ui and Tailwind CSS.
                    Copy the source, own every pixel."
                </p>
            </div>

            <div class="grid grid-cols-1 gap-10 lg:grid-cols-[200px_1fr]">
                <nav class="hidden lg:block">
                    <div class="sticky top-20 space-y-1 border-l border-border pl-4 text-sm">
                        {SECTIONS.iter().map(|(id, label)| {
                            let on_click = scroll_to(id);
                            view! {
                                <a
                                    href="#"
                                    class="block rounded-md px-3 py-1.5 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                                    on:click=on_click
                                >{*label}</a>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </nav>

                <div class="min-w-0 space-y-16">
                    <ComponentSection
                        id="button"
                        title="Button"
                        description="Action triggers with variants and sizes."
                        snippet=BUTTON_SNIPPET
                    >
                        <div class="flex flex-wrap items-center gap-3">
                            <Button>"Default"</Button>
                            <Button variant=ButtonVariant::Secondary>"Secondary"</Button>
                            <Button variant=ButtonVariant::Outline>"Outline"</Button>
                            <Button variant=ButtonVariant::Ghost>"Ghost"</Button>
                            <Button variant=ButtonVariant::Destructive>"Delete"</Button>
                        </div>
                        <div class="mt-4 flex flex-wrap items-center gap-3">
                            <Button size=ButtonSize::Sm>"Small"</Button>
                            <Button>"Default"</Button>
                            <Button size=ButtonSize::Lg>"Large"</Button>
                            <Button size=ButtonSize::Icon>
                                <Icon glyph=Glyph::Search class="h-4 w-4" />
                            </Button>
                        </div>
                    </ComponentSection>

                    <ComponentSection
                        id="badge"
                        title="Badge"
                        description="Short statuses and labels."
                        snippet=BADGE_SNIPPET
                    >
                        <div class="flex flex-wrap items-center gap-3">
                            <Badge>"Default"</Badge>
                            <Badge variant=BadgeVariant::Secondary>"Secondary"</Badge>
                            <Badge variant=BadgeVariant::Outline>"Outline"</Badge>
                            <Badge variant=BadgeVariant::Destructive>"Destructive"</Badge>
                        </div>
                        <div class="mt-4 flex flex-wrap items-center gap-3">
                            <Badge size=BadgeSize::Sm>"Small"</Badge>
                            <Badge>"Default"</Badge>
                            <Badge size=BadgeSize::Lg>"Large"</Badge>
                        </div>
                    </ComponentSection>

                    <ComponentSection
                        id="card"
                        title="Card"
                        description="Contained surfaces for related content."
                        snippet=CARD_SNIPPET
                    >
                        <Card class="max-w-sm">
                            <CardHeader>
                                <CardTitle>"Deployments"</CardTitle>
                                <CardDescription>"Manage your live services"</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <p class="text-sm text-muted-foreground">
                                    "42 services running · 3 pending"
                                </p>
                            </CardContent>
                        </Card>
                    </ComponentSection>

                    <ComponentSection
                        id="input"
                        title="Input"
                        description="Text entry with focus rings and errors."
                        snippet=INPUT_SNIPPET
                    >
                        <div class="flex max-w-sm flex-col gap-3">
                            <Input placeholder="Search packages…" />
                            <Input placeholder="Password" input_type="password" />
                            <Input placeholder="Invalid value" error="Must be at least 3 characters" />
                        </div>
                    </ComponentSection>

                    <ComponentSection
                        id="switch"
                        title="Switch"
                        description="Binary on/off control."
                        snippet=SWITCH_SNIPPET
                    >
                        <div class="flex items-center gap-3">
                            <Switch />
                            <span class="text-sm text-muted-foreground">"Default (off)"</span>
                        </div>
                        <div class="mt-3 flex items-center gap-3">
                            <Switch checked=RwSignal::new(true) />
                            <span class="text-sm text-muted-foreground">"Checked"</span>
                        </div>
                    </ComponentSection>

                    <ComponentSection
                        id="tabs"
                        title="Tabs"
                        description="Switch between related panels."
                        snippet=TABS_SNIPPET
                    >
                        <Tabs default_value="preview">
                            <TabsList>
                                <TabsTrigger value={"preview".to_string()}>"Preview"</TabsTrigger>
                                <TabsTrigger value={"code".to_string()}>"Code"</TabsTrigger>
                            </TabsList>
                            <TabsContent value={"preview".to_string()}>
                                <p class="mt-4 text-sm text-muted-foreground">"Live preview panel"</p>
                            </TabsContent>
                            <TabsContent value={"code".to_string()}>
                                <p class="mt-4 text-sm text-muted-foreground">"Source code panel"</p>
                            </TabsContent>
                        </Tabs>
                    </ComponentSection>

                    <ComponentSection
                        id="accordion"
                        title="Accordion"
                        description="Collapsible content sections."
                        snippet=ACCORDION_SNIPPET
                    >
                        <Accordion class="max-w-md rounded-lg border border-border">
                            <AccordionItem value={"what".to_string()}>
                                <AccordionTrigger>"What is a Plate?"</AccordionTrigger>
                                <AccordionContent>
                                    "A feature module with explicit trait boundaries that registers its routes."
                                </AccordionContent>
                            </AccordionItem>
                            <AccordionItem value={"why".to_string()}>
                                <AccordionTrigger>"Why deterministic?"</AccordionTrigger>
                                <AccordionContent>
                                    "Same input, same output — in production, in tests, on every platform."
                                </AccordionContent>
                            </AccordionItem>
                            <AccordionItem value={"how".to_string()}>
                                <AccordionTrigger>"How do agents help?"</AccordionTrigger>
                                <AccordionContent>
                                    "Spec snapshots and skills make your codebase readable by AI coding partners."
                                </AccordionContent>
                            </AccordionItem>
                        </Accordion>
                    </ComponentSection>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ComponentSection(
    id: &'static str,
    title: &'static str,
    description: &'static str,
    snippet: &'static str,
    children: Children,
) -> impl IntoView {
    let snippet_html = highlight_rust(snippet);
    view! {
        <section id=id class="scroll-mt-24">
            <h2 class="text-2xl font-bold tracking-tight">{title}</h2>
            <p class="mt-1 text-sm text-muted-foreground">{description}</p>

            <div class="mt-4 grid grid-cols-1 gap-4 xl:grid-cols-2">
                <div class="showcase-card p-6">{children()}</div>
                <div class="code-window">
                    <div class="code-window-bar">
                        <span class="traffic-light traffic-light-red"></span>
                        <span class="traffic-light traffic-light-yellow"></span>
                        <span class="traffic-light traffic-light-green"></span>
                        <span class="code-window-tab">{id}.rs</span>
                        <span class="ml-auto">
                            <CopyButton text=snippet.to_string() label="Copy" />
                        </span>
                    </div>
                    <pre class="code-window-body text-left" inner_html=snippet_html></pre>
                </div>
            </div>
        </section>
    }
}
