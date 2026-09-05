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
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-border">
            <div class="page-container py-14">
                <div class="grid grid-cols-1 gap-10 sm:grid-cols-2 lg:grid-cols-4">
                    <div>
                        <div class="flex items-center gap-2 text-lg font-bold">
                            <img src="/logo-64.png" alt="MontRS logo" class="h-5 w-5 rounded" />
                            "MontRS"
                        </div>
                        <p class="mt-3 max-w-xs text-sm text-muted-foreground">
                            "Describe it once. Run it everywhere."
                        </p>
                    </div>
                    <div>
                        <h3 class="text-sm font-semibold">"Framework"</h3>
                        <ul class="mt-3 space-y-2 text-sm text-muted-foreground">
                            <li><a class="transition-colors hover:text-foreground" href="/docs">"Docs"</a></li>
                            <li><a class="transition-colors hover:text-foreground" href="/auth">"Auth"</a></li>
                            <li><a class="transition-colors hover:text-foreground" href="/runtime">"Runtime"</a></li>
                            <li><a class="transition-colors hover:text-foreground" href="/orm">"ORM"</a></li>
                        </ul>
                    </div>
                    <div>
                        <h3 class="text-sm font-semibold">"UI"</h3>
                        <ul class="mt-3 space-y-2 text-sm text-muted-foreground">
                            <li><a class="transition-colors hover:text-foreground" href="/ui/components">"Components"</a></li>
                            <li><a class="transition-colors hover:text-foreground" href="/ui/blocks">"Blocks"</a></li>
                            <li><a class="transition-colors hover:text-foreground" href="/ui/icons">"Icons"</a></li>
                            <li><a class="transition-colors hover:text-foreground" href="/ui/motion">"Motion"</a></li>
                        </ul>
                    </div>
                    <div>
                        <h3 class="text-sm font-semibold">"Community"</h3>
                        <ul class="mt-3 space-y-2 text-sm text-muted-foreground">
                            <li>
                                <a class="transition-colors hover:text-foreground" href="https://github.com/montrs/montrs" target="_blank" rel="noopener noreferrer">
                                    "GitHub"
                                </a>
                            </li>
                            <li>
                                <a class="transition-colors hover:text-foreground" href="https://github.com/montrs/montrs" target="_blank" rel="noopener noreferrer">
                                    "Documentation"
                                </a>
                            </li>
                            <li>"Apache-2.0 / MIT"</li>
                        </ul>
                    </div>
                </div>
                <div class="mt-10 flex flex-col items-center justify-between gap-4 border-t border-border pt-8 text-sm text-muted-foreground sm:flex-row">
                    <p>"© 2026 MontRS — Apache-2.0 / MIT"</p>
                    <a
                        class="font-mono text-xs transition-colors hover:text-foreground"
                        href="https://github.com/montrs/montrs"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        "github.com/montrs/montrs"
                    </a>
                </div>
            </div>
        </footer>
    }
}
