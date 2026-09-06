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

use crate::pages::*;
use async_trait::async_trait;
use leptos::prelude::*;
use montrs_core::*;

// ---------------------------------------------------------------------------
// RouteView wrappers for each page component
// ---------------------------------------------------------------------------

pub struct HomeView;
impl RouteView for HomeView {
    fn render(&self) -> impl IntoView {
        view! { <Home /> }
    }
}

pub struct UiView;
impl RouteView for UiView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::Ui /> }
    }
}

pub struct IconsView;
impl RouteView for IconsView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::Icons /> }
    }
}

pub struct ComponentsView;
impl RouteView for ComponentsView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::Components /> }
    }
}

pub struct BlocksView;
impl RouteView for BlocksView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::Blocks /> }
    }
}

pub struct MotionView;
impl RouteView for MotionView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::Motion /> }
    }
}

pub struct DocsView;
impl RouteView for DocsView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::Docs /> }
    }
}

pub struct AuthView;
impl RouteView for AuthView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::Auth /> }
    }
}

pub struct RuntimeView;
impl RouteView for RuntimeView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::Runtime /> }
    }
}

pub struct AiKitView;
impl RouteView for AiKitView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::AiKit /> }
    }
}

pub struct OrmView;
impl RouteView for OrmView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::Orm /> }
    }
}

pub struct FoundationsView;
impl RouteView for FoundationsView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::Foundations /> }
    }
}

pub struct ThemesView;
impl RouteView for ThemesView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::Themes /> }
    }
}

pub struct BackgroundsView;
impl RouteView for BackgroundsView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::Backgrounds /> }
    }
}

pub struct TemplatesView;
impl RouteView for TemplatesView {
    fn render(&self) -> impl IntoView {
        view! { <crate::pages::Templates /> }
    }
}

// ---------------------------------------------------------------------------
// MontRS Routes
// ---------------------------------------------------------------------------

view_route! { HomeRoute, "/", HomeView }

// UI section (components, blocks, icons, motion)
view_route! { UiRoute, "/ui", UiView }
view_route! { ComponentsRoute, "/ui/components", ComponentsView }
view_route! { BlocksRoute, "/ui/blocks", BlocksView }
view_route! { IconsRoute, "/ui/icons", IconsView }
view_route! { MotionRoute, "/ui/motion", MotionView }

// Framework sections
view_route! { DocsRoute, "/docs", DocsView }
view_route! { AuthRoute, "/auth", AuthView }
view_route! { RuntimeRoute, "/runtime", RuntimeView }
view_route! { AiKitRoute, "/ai", AiKitView }
view_route! { OrmRoute, "/orm", OrmView }
view_route! { FoundationsRoute, "/foundations", FoundationsView }

// Themes / backgrounds / templates
view_route! { ThemesRoute, "/ui/themes", ThemesView }
view_route! { BackgroundsRoute, "/ui/backgrounds", BackgroundsView }
view_route! { TemplatesRoute, "/templates", TemplatesView }

// ---------------------------------------------------------------------------
// Website Plate
// ---------------------------------------------------------------------------

pub struct WebsitePlate;

#[async_trait]
impl<C: AppConfig + 'static> Plate<C> for WebsitePlate {
    fn name(&self) -> &'static str {
        "website"
    }

    fn description(&self) -> &'static str {
        "MontRS website — montrs.com"
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn init(
        &self,
        _ctx: &mut PlateContext<C>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    fn register_routes(&self, router: &mut Router<C>) {
        router.register(HomeRoute);
        router.register(UiRoute);
        router.register(ComponentsRoute);
        router.register(BlocksRoute);
        router.register(IconsRoute);
        router.register(MotionRoute);
        router.register(ThemesRoute);
        router.register(BackgroundsRoute);
        router.register(TemplatesRoute);
        router.register(DocsRoute);
        router.register(AuthRoute);
        router.register(RuntimeRoute);
        router.register(AiKitRoute);
        router.register(OrmRoute);
        router.register(FoundationsRoute);
    }
}
