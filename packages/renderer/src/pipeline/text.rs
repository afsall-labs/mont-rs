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

#[cfg(feature = "text")]
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};

#[cfg(feature = "text")]
pub struct TextPipeline {
    font_system: FontSystem,
    _swash_cache: SwashCache,
    buffers: Vec<(f32, f32, String, f32, [f32; 4])>,
    viewport: Option<(u32, u32)>,
}

#[cfg(not(feature = "text"))]
pub struct TextPipeline;

#[cfg(feature = "text")]
impl TextPipeline {
    pub fn new(
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> Self {
        let font_system = FontSystem::new();
        let _swash_cache = SwashCache::new();
        Self {
            font_system,
            _swash_cache,
            buffers: Vec::new(),
            viewport: None,
        }
    }

    pub fn push(
        &mut self,
        x: f32,
        y: f32,
        text: &str,
        font_size: f32,
        color: [f32; 4],
    ) {
        self.buffers
            .push((x, y, text.to_string(), font_size, color));
    }

    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.viewport = Some((width, height));
    }

    pub fn flush(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _encoder: &mut wgpu::CommandEncoder,
        _view: &wgpu::TextureView,
    ) {
        if self.buffers.is_empty() {
            return;
        }

        let (_vp_w, _vp_h) = self.viewport.unwrap_or((800, 600));

        for (x, y, text, font_size, _color) in &self.buffers {
            let mut buffer = Buffer::new(
                &mut self.font_system,
                Metrics::new(font_size.round(), font_size.round() * 1.2),
            );
            buffer.set_size(
                &mut self.font_system,
                Some(_vp_w as f32),
                Some(_vp_h as f32),
            );
            buffer.set_text(
                &mut self.font_system,
                text,
                Attrs::new(),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system, false);

            let _ = x;
            let _ = y;
        }

        self.buffers.clear();
    }

    pub fn resize(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        self.viewport = Some((width, height));
    }
}

#[cfg(not(feature = "text"))]
impl TextPipeline {
    pub fn new(_device: &(), _queue: &(), _format: ()) -> Self {
        Self
    }

    pub fn push(
        &mut self,
        _x: f32,
        _y: f32,
        _text: &str,
        _font_size: f32,
        _color: [f32; 4],
    ) {
    }

    pub fn set_viewport(&mut self, _width: u32, _height: u32) {}

    pub fn flush(
        &mut self,
        _device: &(),
        _queue: &(),
        _encoder: &mut (),
        _view: &(),
    ) {
    }

    pub fn resize(
        &mut self,
        _device: &(),
        _queue: &(),
        _width: u32,
        _height: u32,
    ) {
    }
}
