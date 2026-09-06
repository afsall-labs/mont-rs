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

use crate::highlight::highlight_rust;
use leptos::prelude::*;
use montrs_icons::*;
use montrs_ui::prelude::*;

const ORM_SNIPPET: &str = r#"use montrs_orm::*;
use montrs_orm::backend::postgres::Postgres;

let db = Postgres::connect("postgres://localhost/montrs").await?;

let users = db
    .query("SELECT * FROM users WHERE org_id = $1")
    .bind(org_id)
    .map(User::from_row)
    .await?;

db.transaction(|tx| async {
    tx.execute("INSERT INTO audits (user_id, action) VALUES ($1, $2)", ...).await?;
    Ok(())
}).await?;
"#;

#[component]
pub fn Orm() -> impl IntoView {
    let items = [
        (
            Glyph::Terminal,
            "SQL-first",
            "Write real SQL with typed rows. No magic query builders hiding \
             your data access.",
        ),
        (
            Glyph::Table2,
            "Backend-agnostic",
            "Postgres, SQLite, and any backend through a single Connection \
             trait.",
        ),
        (
            Glyph::Workflow,
            "Transactions",
            "Type-safe transactions and migrations as part of the same spec.",
        ),
        (
            Glyph::ShieldCheck,
            "Deterministic",
            "Your queries behave identically in tests and in production.",
        ),
    ];

    view! {
        <div class="page-container py-12">
            <div class="mb-10">
                <h1 class="text-3xl font-bold tracking-tight">"ORM"</h1>
                <p class="mt-2 max-w-2xl text-muted-foreground">
                    "A SQL-first, backend-agnostic database abstraction —
                    where SQL stays SQL."
                </p>
            </div>

            <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
                {items.into_iter().map(|(icon, title, desc)| view! {
                    <div class="showcase-card reveal p-6">
                        <Icon glyph=icon class="h-6 w-6 text-primary" />
                        <h3 class="mt-4 font-semibold">{title}</h3>
                        <p class="mt-2 text-sm leading-6 text-muted-foreground">{desc}</p>
                    </div>
                }).collect::<Vec<_>>()}
            </div>

            <div class="mt-10">
                <div class="code-window max-w-2xl">
                    <div class="code-window-bar">
                        <span class="traffic-light traffic-light-red"></span>
                        <span class="traffic-light traffic-light-yellow"></span>
                        <span class="traffic-light traffic-light-green"></span>
                        <span class="code-window-tab">"users.rs"</span>
                    </div>
                    <pre class="code-window-body text-left" inner_html=highlight_rust(ORM_SNIPPET)></pre>
                </div>
            </div>
        </div>
    }
}
