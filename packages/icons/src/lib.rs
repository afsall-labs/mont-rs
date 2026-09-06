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

//! montrs-icons: Lucide icons for MontRS applications.
//!
//! Provides 1600+ icons as Leptos components, grouped into 42 category features
//! for selective compilation. Prefer the per-icon convenience components
//! (e.g. [`SearchIcon`]) over the generic [`Icon`] component for static usage.
//! Use the [`glyph!`] macro with the generic [`Icon`] when you need a dynamic
//! or reactive glyph.

pub mod collections;
pub mod glyph;
pub mod glyph_impl;
pub mod icon;
pub mod registry;

#[cfg(feature = "animated")]
pub mod animated;

#[cfg(feature = "animated")]
pub use animated::{AnimatedIcon, AnimatedSvg, AnimationProfile, animation_profile};
pub use collections::{CollectedGlyph, Collection, CollectionInfo};
pub use glyph::Glyph;
pub use icon::{CustomIcon, Icon};
pub use registry::*;

/// Re-export strum traits for iterating/looking up icons.
pub mod strum {
    pub use ::strum::{EnumProperty, IntoEnumIterator};
}

/// Shorthand for constructing a [`Glyph`] variant.
///
/// This lets you avoid typing `Glyph::` when using the generic [`Icon`]
/// component:
///
/// ```rust,ignore
/// use montrs_icons::{glyph, Icon};
///
/// view! { <Icon glyph=glyph!(Search) /> }
/// ```
///
/// This is equivalent to `Glyph::Search`. Prefer the per-icon convenience
/// components (e.g. `<SearchIcon />`) for static usage; reach for this
/// macro when you need a dynamic or reactive glyph.
#[macro_export]
macro_rules! glyph {
    ($name:ident) => {
        $crate::Glyph::$name
    };
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
