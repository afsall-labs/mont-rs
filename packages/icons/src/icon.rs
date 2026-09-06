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

use crate::glyph::Glyph;
use leptos::{prelude::*, text_prop::TextProp};

pub const DEFAULT_SIZE: &str = "24";
pub const DEFAULT_FILL: &str = "none";
pub const DEFAULT_STROKE: &str = "currentColor";
pub const DEFAULT_STROKE_WIDTH: &str = "1.5";

#[component]
pub fn Icon(
    #[prop(into)] glyph: Signal<Glyph>,
    #[prop(into, optional)] class: Option<TextProp>,
    #[prop(into, optional)] size: Option<TextProp>,
    #[prop(into, optional)] fill: Option<TextProp>,
    #[prop(into, optional)] stroke: Option<TextProp>,
    #[prop(into, optional)] stroke_width: Option<TextProp>,
    /// Override the view-box (defaults to "0 0 24 24").
    #[prop(into, optional)] viewbox: Option<TextProp>,
) -> impl IntoView {
    let svg = TextProp::from(move || glyph.get().svg());
    render_svg(svg, class, size, fill, stroke, stroke_width, viewbox)
}

#[component]
pub fn CustomIcon(
    #[prop(into)] svg: TextProp,
    #[prop(into, optional)] class: Option<TextProp>,
    #[prop(into, optional)] size: Option<TextProp>,
    #[prop(into, optional)] fill: Option<TextProp>,
    #[prop(into, optional)] stroke: Option<TextProp>,
    #[prop(into, optional)] stroke_width: Option<TextProp>,
    /// Override the view-box (defaults to "0 0 24 24").
    #[prop(into, optional)] viewbox: Option<TextProp>,
) -> impl IntoView {
    render_svg(svg, class, size, fill, stroke, stroke_width, viewbox)
}

pub fn render_svg(
    svg: TextProp,
    class: Option<TextProp>,
    size: Option<TextProp>,
    fill: Option<TextProp>,
    stroke: Option<TextProp>,
    stroke_width: Option<TextProp>,
    viewbox: Option<TextProp>,
) -> impl IntoView {
    let class = class.unwrap_or_else(|| "".into());
    let size = size.unwrap_or_else(|| DEFAULT_SIZE.into());
    let size2 = size.clone();
    let fill = fill.unwrap_or_else(|| DEFAULT_FILL.into());
    let stroke = stroke.unwrap_or_else(|| DEFAULT_STROKE.into());
    let stroke_width =
        stroke_width.unwrap_or_else(|| DEFAULT_STROKE_WIDTH.into());
    let viewbox = viewbox.unwrap_or_else(|| "0 0 24 24".into());

    view! {
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class=move || class.get()
          width=move || size.get()
          height=move || size2.get()
          viewBox=move || viewbox.get()
          fill=move || fill.get()
          stroke=move || stroke.get()
          stroke-width=move || stroke_width.get()
          stroke-linecap="round"
          stroke-linejoin="round"
          inner_html=move || svg.get()
        />
    }
}
