// Ø¨ÙØ³Ù’Ù…Ù Ø§Ù„Ù„ÙŽÙ‘Ù‡Ù Ø§Ù„Ø±ÙŽÙ‘Ø­Ù’Ù…ÙŽÙ†Ù Ø§Ù„Ø±ÙŽÙ‘Ø­ÙÙŠÙ…
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

//! Additional icon collections beyond the built-in Lucide set.
//!
//! Collections are license-safe (MIT / Apache-2.0 only), fetched from the
//! upstream GitHub repositories by `montrs-icons-codegen`, and embedded as
//! static data tables behind per-collection Cargo features. See
//! THIRD_PARTY_NOTICES.md at the repo root.

use crate::glyph::Glyph;

/// A lightweight, render-agnostic glyph from any collection.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CollectedGlyph {
    /// Display name (kebab-case for collections, PascalCase for Lucide).
    pub name: &'static str,
    /// Inner SVG markup (the child elements, no `<svg>` wrapper).
    pub svg: &'static str,
    /// View-box string, e.g. "0 0 24 24".
    pub viewbox: &'static str,
    /// Root fill ("none" for stroke-based sets, "currentColor" for fill sets).
    pub fill: &'static str,
    /// Root stroke ("currentColor" for stroke sets, "none" for fill sets).
    pub stroke: &'static str,
}

impl CollectedGlyph {
    pub fn pascal_name(&self) -> String {
        self.name
            .split('-')
            .map(|part| {
                let mut c = part.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            })
            .collect::<String>()
    }
}

/// Catalog metadata for a collection (generated).
#[derive(Clone, Copy)]
pub struct CollectionInfo {
    pub key: &'static str,
    pub label: &'static str,
    pub license: &'static str,
    pub source: &'static str,
}

/// An icon collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collection {
    Lucide,
    Radix,
    Tabler,
    Iconoir,
    Phosphor,
    Mdi,
}

impl Collection {
    /// Every locally available collection (Lucide + feature-gated extras).
    pub const ALL: &'static [Collection] = &[
        Collection::Lucide,
        #[cfg(feature = "col-radix")]
        Collection::Radix,
        #[cfg(feature = "col-tabler")]
        Collection::Tabler,
        #[cfg(feature = "col-iconoir")]
        Collection::Iconoir,
        #[cfg(feature = "col-phosphor")]
        Collection::Phosphor,
        #[cfg(feature = "col-mdi")]
        Collection::Mdi,
    ];

    pub fn label(self) -> &'static str {
        self.info().map(|i| i.label).unwrap_or("Lucide")
    }

    /// Collection catalog; unknown collections resolve to Lucide.
    pub fn info(self) -> Option<CollectionInfo> {
        Collection::catalog()
            .into_iter()
            .find(|i| i.key == self.key())
    }

    pub fn key(self) -> &'static str {
        match self {
            Collection::Lucide => "lucide",
            Collection::Radix => "radix",
            Collection::Tabler => "tabler",
            Collection::Iconoir => "iconoir",
            Collection::Phosphor => "phosphor",
            Collection::Mdi => "mdi",
        }
    }

    /// Rendering style: "stroke" sets draw with strokes; "fill" sets draw
    /// with filled paths and should ignore stroke width/color overrides.
    pub fn style(self) -> &'static str {
        match self {
            Collection::Lucide | Collection::Radix | Collection::Tabler | Collection::Iconoir => {
                "stroke"
            }
            Collection::Phosphor | Collection::Mdi => "fill",
        }
    }

    /// Stroke width that suits this collection's view-box (fill sets return
    /// their captured value, which callers can ignore).
    pub fn default_stroke_width(self) -> f64 {
        match self {
            Collection::Tabler => 2.0,
            Collection::Radix => 1.2,
            _ => 1.5,
        }
    }

    /// Resolve a collection by its key string (unknown keys â†’ `None`).
    pub fn from_key(key: &str) -> Option<Collection> {
        Collection::ALL.iter().copied().find(|c| c.key() == key)
    }

    /// Look up a single glyph by name (kebab or PascalCase).
    pub fn glyph(self, name: &str) -> Option<CollectedGlyph> {
        let pascal = name.to_string();
        self.icons()
            .into_iter()
            .find(|g| g.name == name || g.pascal_name() == pascal)
    }

    pub fn catalog() -> Vec<CollectionInfo> {
        crate::collections::catalog::COLLECTION_LIST.to_vec()
    }

    /// All icons in this collection (Lucide is synthesized from the `Glyph`
    /// enum; the rest come from generated static tables).
    #[allow(unreachable_patterns)]
    pub fn icons(self) -> Vec<CollectedGlyph> {
        match self {
            Collection::Lucide => Glyph::find("")
                .into_iter()
                .map(|g| CollectedGlyph {
                    name: g.name(),
                    svg: g.svg(),
                    viewbox: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                })
                .collect(),
            #[cfg(feature = "col-radix")]
            Collection::Radix => table(&crate::collections::data::RADIX_ICONS),
            #[cfg(feature = "col-tabler")]
            Collection::Tabler => table(&crate::collections::data::TABLER_ICONS),
            #[cfg(feature = "col-iconoir")]
            Collection::Iconoir => table(&crate::collections::data::ICONOIR_ICONS),
            #[cfg(feature = "col-phosphor")]
            Collection::Phosphor => table(&crate::collections::data::PHOSPHOR_ICONS),
            #[cfg(feature = "col-mdi")]
            Collection::Mdi => table(&crate::collections::data::MDI_ICONS),
            _ => Vec::new(),
        }
    }

    /// Total glyph count for this collection (`0` if its feature is off).
    #[allow(unreachable_patterns)]
    pub fn count(self) -> usize {
        match self {
            Collection::Lucide => Glyph::count(),
            #[cfg(feature = "col-radix")]
            Collection::Radix => crate::collections::data::RADIX_ICONS.len(),
            #[cfg(feature = "col-tabler")]
            Collection::Tabler => crate::collections::data::TABLER_ICONS.len(),
            #[cfg(feature = "col-iconoir")]
            Collection::Iconoir => crate::collections::data::ICONOIR_ICONS.len(),
            #[cfg(feature = "col-phosphor")]
            Collection::Phosphor => crate::collections::data::PHOSPHOR_ICONS.len(),
            #[cfg(feature = "col-mdi")]
            Collection::Mdi => crate::collections::data::MDI_ICONS.len(),
            _ => 0,
        }
    }

    /// Icons matching a plain substring on the name (search helper).
    pub fn find(self, filter: &str) -> Vec<CollectedGlyph> {
        let all = self.icons();
        if filter.is_empty() {
            return all;
        }
        let f = filter.to_lowercase();
        all.into_iter()
            .filter(|g| g.name.to_lowercase().contains(&f))
            .collect()
    }
}

fn table(src: &'static [CollectedGlyph]) -> Vec<CollectedGlyph> {
    src.to_vec()
}

pub mod catalog {
    //! Generated collection metadata.
    use super::super::CollectionInfo;
    include!("catalog.rs");
}

pub mod data {
    //! Generated static glyph tables.
    include!("data.rs");
}
