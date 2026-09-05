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

const AUTH_SNIPPET: &str = r#"use montrs_auth::prelude::*;

let auth = Auth::builder()
    .with(EmailPassword::default())
    .with(OAuth::github())
    .with(TwoFactor::totp())
    .with(Passkey::default())
    .with(ApiKey::default())
    .build()?;

// Middleware on any route
router.get("/protected", auth.require(Role::Admin), handler);
"#;

#[component]
pub fn Auth() -> impl IntoView {
    let features = [
        (
            Glyph::Mail,
            "Email & Password",
            "Verification, password reset, and one-time tokens.",
        ),
        (
            Glyph::Globe,
            "OAuth Providers",
            "Google, GitHub, and any generic OAuth 2.0 / OIDC provider.",
        ),
        (
            Glyph::ShieldCheck,
            "Two-Factor",
            "TOTP, recovery codes, phone OTP, and 2FA challenge flows.",
        ),
        (
            Glyph::KeySquare,
            "Passkeys",
            "WebAuthn passkey authentication with device authorization.",
        ),
        (
            Glyph::KeyRound,
            "API Keys",
            "Scoped, rotating API keys for service-to-service auth.",
        ),
        (
            Glyph::Workflow,
            "Sessions & SSO",
            "Multi-session management, SAML/SSO, and session revocation.",
        ),
        (
            Glyph::Users,
            "Organizations & RBAC",
            "Teams, roles, permissions, and SCIM provisioning.",
        ),
        (
            Glyph::Eye,
            "Zero-Trust Extras",
            "Device codes, magic links, captcha, and breached-password checks.",
        ),
    ];

    view! {
        <div class="page-container py-12">
            <div class="mb-10">
                <h1 class="text-3xl font-bold tracking-tight">"Auth"</h1>
                <p class="mt-2 max-w-2xl text-muted-foreground">
                    "A complete, plugin-based authentication system — email/password,
                    OAuth, 2FA, passkeys, sessions, and RBAC — composed behind one trait."
                </p>
            </div>

            <div class="mb-10 flex flex-wrap gap-x-8 gap-y-2 font-mono text-sm text-muted-foreground">
                <span>"30+"</span><span class="text-foreground">"plugins"</span>
                <span>"· "</span><span>"10+ "</span><span class="text-foreground">"OAuth providers"</span>
                <span>"· "</span><span>"full "</span><span class="text-foreground">"RBAC"</span>
                <span>"· "</span><span>"agent-"></span><span class="text-foreground">"native"</span>
            </div>

            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                {features.into_iter().map(|(icon, title, desc)| view! {
                    <div class="showcase-card reveal p-6">
                        <div class="flex items-center gap-3">
                            <span class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-primary">
                                <Icon glyph=icon class="h-5 w-5" />
                            </span>
                            <h3 class="font-semibold">{title}</h3>
                        </div>
                        <p class="mt-3 text-sm leading-6 text-muted-foreground">{desc}</p>
                    </div>
                }).collect::<Vec<_>>()}
            </div>

            <div class="mt-10 grid grid-cols-1 gap-8 lg:grid-cols-2">
                <div class="code-window">
                    <div class="code-window-bar">
                        <span class="traffic-light traffic-light-red"></span>
                        <span class="traffic-light traffic-light-yellow"></span>
                        <span class="traffic-light traffic-light-green"></span>
                        <span class="code-window-tab">"auth.rs"</span>
                    </div>
                    <pre class="code-window-body text-left" inner_html=highlight_rust(AUTH_SNIPPET)></pre>
                </div>
                <div class="flex flex-col justify-center space-y-4 text-sm text-muted-foreground">
                    <div class="flex items-center gap-3 rounded-md border border-border px-4 py-3">
                        <Icon glyph=Glyph::Check class="h-4 w-4 shrink-0 text-success" />
                        "Rate-limited by default (Governor limiter)"
                    </div>
                    <div class="flex items-center gap-3 rounded-md border border-border px-4 py-3">
                        <Icon glyph=Glyph::Check class="h-4 w-4 shrink-0 text-success" />
                        "Email template hooks, i18n, and custom plugins"
                    </div>
                    <div class="flex items-center gap-3 rounded-md border border-border px-4 py-3">
                        <Icon glyph=Glyph::Check class="h-4 w-4 shrink-0 text-success" />
                        "Plugin gating, admin roles, and scoped access policies"
                    </div>
                </div>
            </div>
        </div>
    }
}
