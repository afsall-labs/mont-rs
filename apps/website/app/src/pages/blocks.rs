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

use crate::{
    blocks::*,
    copy::CopyButton,
    highlight::{highlight_rust, strip_license},
};
use leptos::prelude::*;
use montrs_icons::*;
use montrs_ui::prelude::*;

#[component]
pub fn Blocks() -> impl IntoView {
    view! {
        <div class="page-container py-12">
            <div class="mb-10">
                <h1 class="text-3xl font-bold tracking-tight">"Blocks"</h1>
                <p class="mt-2 max-w-2xl text-muted-foreground">
                    "Pre-built UI sections built from real MontRS Plates.
                    Copy, paste, and customize — no generators, no magic."
                </p>
                <div class="terminal mt-6 flex max-w-xl items-center justify-between gap-4">
                    <span>
                        <span class="terminal-prompt">"$"</span>
                        " montrs serve"
                    </span>
                    <CopyButton text="montrs serve".to_string() label="Copy" />
                </div>
            </div>

            <SectionTitle>"FAQ"</SectionTitle>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
                <BlockCard name="faq-01.rs" source=include_str!("../blocks/faq/faq01.rs")>
                    <Faq01 />
                </BlockCard>
                <BlockCard name="faq-02.rs" source=include_str!("../blocks/faq/faq02.rs")>
                    <Faq02 />
                </BlockCard>
                <BlockCard name="faq-03.rs" source=include_str!("../blocks/faq/faq03.rs")>
                    <Faq03 />
                </BlockCard>
            </div>

            <SectionTitle>"Footers"</SectionTitle>
            <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
                <BlockCard name="footer-01.rs" source=include_str!("../blocks/footer/footer01.rs")>
                    <Footer01 />
                </BlockCard>
                <BlockCard name="footer-02.rs" source=include_str!("../blocks/footer/footer02.rs")>
                    <Footer02 />
                </BlockCard>
                <BlockCard name="footer-03.rs" source=include_str!("../blocks/footer/footer03.rs")>
                    <Footer03 />
                </BlockCard>
                <BlockCard name="footer-04.rs" source=include_str!("../blocks/footer/footer04.rs")>
                    <Footer04 />
                </BlockCard>
                <BlockCard name="footer-05.rs" source=include_str!("../blocks/footer/footer05.rs")>
                    <Footer05 />
                </BlockCard>
                <BlockCard name="footer-logos.rs" source=include_str!("../blocks/footer/footer_logos.rs")>
                    <FooterLogos />
                </BlockCard>
            </div>

            <SectionTitle>"Headers"</SectionTitle>
            <BlockCard name="header-01.rs" source=include_str!("../blocks/header/header01.rs")>
                <Header01 />
            </BlockCard>

            <SectionTitle>"Integrations"</SectionTitle>
            <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
                <BlockCard name="integration-01.rs" source=include_str!("../blocks/integration/integration01.rs")>
                    <Integration01 />
                </BlockCard>
                <BlockCard name="integration-02.rs" source=include_str!("../blocks/integration/integration02.rs")>
                    <Integration02 />
                </BlockCard>
                <BlockCard name="integration-03.rs" source=include_str!("../blocks/integration/integration03.rs")>
                    <Integration03 />
                </BlockCard>
                <BlockCard name="integration-04.rs" source=include_str!("../blocks/integration/integration04.rs")>
                    <Integration04 />
                </BlockCard>
                <BlockCard name="integration-05.rs" source=include_str!("../blocks/integration/integration05.rs")>
                    <Integration05 />
                </BlockCard>
                <BlockCard name="integration-06.rs" source=include_str!("../blocks/integration/integration06.rs")>
                    <Integration06 />
                </BlockCard>
                <BlockCard name="integration-07.rs" source=include_str!("../blocks/integration/integration07.rs")>
                    <Integration07 />
                </BlockCard>
            </div>

            <SectionTitle>"Login"</SectionTitle>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                <BlockCard name="login-01.rs" source=include_str!("../blocks/login/login01.rs")>
                    <Login01 />
                </BlockCard>
                <BlockCard name="login-02.rs" source=include_str!("../blocks/login/login02.rs")>
                    <Login02 />
                </BlockCard>
                <BlockCard name="login-03.rs" source=include_str!("../blocks/login/login03.rs")>
                    <Login03 />
                </BlockCard>
                <BlockCard name="login-04.rs" source=include_str!("../blocks/login/login04.rs")>
                    <Login04 />
                </BlockCard>
            </div>

            <SectionTitle>"Sidenav"</SectionTitle>
            <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
                <BlockCard name="sidenav-01.rs" source=include_str!("../blocks/sidenav/sidenav01.rs")>
                    <Sidenav01 />
                </BlockCard>
                <BlockCard name="sidenav-02.rs" source=include_str!("../blocks/sidenav/sidenav02.rs")>
                    <Sidenav02 />
                </BlockCard>
                <BlockCard name="sidenav-03.rs" source=include_str!("../blocks/sidenav/sidenav03.rs")>
                    <Sidenav03 />
                </BlockCard>
                <BlockCard name="sidenav-04.rs" source=include_str!("../blocks/sidenav/sidenav04.rs")>
                    <Sidenav04 />
                </BlockCard>
                <BlockCard name="sidenav-05.rs" source=include_str!("../blocks/sidenav/sidenav05.rs")>
                    <Sidenav05 />
                </BlockCard>
                <BlockCard name="sidenav-06.rs" source=include_str!("../blocks/sidenav/sidenav06.rs")>
                    <Sidenav06 />
                </BlockCard>
                <BlockCard name="sidenav-07.rs" source=include_str!("../blocks/sidenav/sidenav07.rs")>
                    <Sidenav07 />
                </BlockCard>
                <BlockCard name="sidenav-08.rs" source=include_str!("../blocks/sidenav/sidenav08.rs")>
                    <Sidenav08 />
                </BlockCard>
                <BlockCard name="sidenav-09.rs" source=include_str!("../blocks/sidenav/sidenav09.rs")>
                    <Sidenav09 />
                </BlockCard>
                <BlockCard name="sidenav-10.rs" source=include_str!("../blocks/sidenav/sidenav10.rs")>
                    <Sidenav10 />
                </BlockCard>
                <BlockCard name="sidenav-11.rs" source=include_str!("../blocks/sidenav/sidenav11.rs")>
                    <Sidenav11 />
                </BlockCard>
                <BlockCard name="sidenav-inset-right.rs" source=include_str!("../blocks/sidenav/sidenav_inset_right.rs")>
                    <SidenavInsetRight />
                </BlockCard>
                <BlockCard name="sidenav-routes.rs" source=include_str!("../blocks/sidenav/sidenav_routes.rs")>
                    <SidenavRoutes />
                </BlockCard>
                <BlockCard name="sidenav-routes-selector.rs" source=include_str!("../blocks/sidenav/sidenav_routes_selector.rs")>
                    <SidenavRoutesSelector />
                </BlockCard>
                <BlockCard name="sidenav-routes-simplified.rs" source=include_str!("../blocks/sidenav/sidenav_routes_simplified.rs")>
                    <SidenavRoutesSimplified />
                </BlockCard>
            </div>
        </div>
    }
}

#[component]
fn SectionTitle(children: Children) -> impl IntoView {
    view! {
        <h2 class="mb-4 mt-12 text-xl font-semibold tracking-tight first:mt-0">
            {children()}
        </h2>
    }
}

#[component]
fn BlockCard(
    name: &'static str,
    source: &'static str,
    children: Children,
) -> impl IntoView {
    let show_code = RwSignal::new(false);
    // Like shadcn/ui, the copied snippet is the implementation only — the
    // SPDX license header is stripped before highlighting.
    let code_html = highlight_rust(strip_license(source));
    let cli = "montrs new".to_string();

    view! {
        <div class="showcase-card flex flex-col">
            <div class="flex items-center justify-between gap-2 border-b border-border px-4 py-2.5">
                <span class="font-mono text-xs text-muted-foreground">{name}</span>
                <div class="flex items-center gap-2">
                    <button
                        type="button"
                        class=move || {
                            let base = "inline-flex items-center gap-1.5 rounded-md border border-border px-2.5 py-1 text-xs font-medium transition-colors";
                            if show_code.get() {
                                format!("{base} border-primary bg-primary/10 text-primary")
                            } else {
                                format!("{base} text-muted-foreground hover:bg-accent hover:text-foreground")
                            }
                        }
                        on:click=move |_| show_code.update(|v| *v = !*v)
                    >
                        <Icon glyph=Glyph::CodeXml class="h-3.5 w-3.5" />
                        {move || if show_code.get() { "Preview" } else { "Code" }}
                    </button>
                    <CopyButton text=cli label="Copy" />
                </div>
            </div>
            <div class="flex-1 p-4">{children()}</div>
            <pre
                class=move || {
                    let base = "max-h-96 overflow-auto border-t border-border bg-background p-4 font-mono text-xs leading-6";
                    if show_code.get() { base.to_string() } else { format!("{base} hidden") }
                }
                inner_html=code_html
            ></pre>
        </div>
    }
}
