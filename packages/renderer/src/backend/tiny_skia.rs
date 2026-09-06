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
    Frame, Paint, Path, Point, Quad, Rect, Renderer, Shape, Stroke, Viewport,
};
use std::collections::HashMap;
use tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Paint as SkPaint, Path as SkPath,
    PathBuilder, Pixmap, Rect as SkRect, Stroke as SkStroke, Transform,
};

struct LayerState {
    alpha: f32,
    pixmap: Pixmap,
}

pub struct SkiaRenderer {
    pixmap: Pixmap,
    width: u32,
    height: u32,
    clip_rect: Option<Rect>,
    layer_stack: Vec<LayerState>,
    image_cache: HashMap<u64, Pixmap>,
    next_image_id: u64,
}

impl SkiaRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        let pixmap = Pixmap::new(width.max(1), height.max(1))
            .expect("Failed to create pixmap");
        Self {
            pixmap,
            width,
            height,
            clip_rect: None,
            layer_stack: Vec::new(),
            image_cache: HashMap::new(),
            next_image_id: 1,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.pixmap = Pixmap::new(self.width, self.height)
            .expect("Failed to resize pixmap");
    }
}

impl Renderer for SkiaRenderer {
    fn begin(&mut self, viewport: &Viewport) {
        let (w, h) = viewport.physical_size();
        self.resize(w, h);
        self.pixmap.fill(Color::from_rgba8(26, 26, 26, 255));
    }

    fn fill_quad(&mut self, quad: &Quad, paint: &Paint) {
        let color = Color::from_rgba8(
            (paint.color.r * 255.0) as u8,
            (paint.color.g * 255.0) as u8,
            (paint.color.b * 255.0) as u8,
            (paint.color.a * 255.0) as u8,
        );

        let rect = match SkRect::from_xywh(
            quad.rect.x,
            quad.rect.y,
            quad.rect.width,
            quad.rect.height,
        ) {
            Some(r) => r,
            None => return,
        };

        let mut sk_paint = SkPaint::default();
        sk_paint.set_color(color);
        sk_paint.anti_alias = paint.anti_alias;

        if quad.corner_radius > 0.0 {
            let r = quad.corner_radius;
            let mut pb = PathBuilder::new();
            pb.move_to(rect.left() + r, rect.top());
            pb.line_to(rect.right() - r, rect.top());
            pb.quad_to(rect.right(), rect.top(), rect.right(), rect.top() + r);
            pb.line_to(rect.right(), rect.bottom() - r);
            pb.quad_to(
                rect.right(),
                rect.bottom(),
                rect.right() - r,
                rect.bottom(),
            );
            pb.line_to(rect.left() + r, rect.bottom());
            pb.quad_to(
                rect.left(),
                rect.bottom(),
                rect.left(),
                rect.bottom() - r,
            );
            pb.line_to(rect.left(), rect.top() + r);
            pb.quad_to(rect.left(), rect.top(), rect.left() + r, rect.top());
            pb.close();
            if let Some(path) = pb.finish() {
                self.pixmap.fill_path(
                    &path,
                    &sk_paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
        } else {
            self.pixmap
                .fill_rect(rect, &sk_paint, Transform::identity(), None);
        }
    }

    fn fill_path(&mut self, path: &Path, paint: &Paint) {
        let sk_path = path_to_skia(path);
        let color = Color::from_rgba8(
            (paint.color.r * 255.0) as u8,
            (paint.color.g * 255.0) as u8,
            (paint.color.b * 255.0) as u8,
            (paint.color.a * 255.0) as u8,
        );
        let mut sk_paint = SkPaint::default();
        sk_paint.set_color(color);
        sk_paint.anti_alias = paint.anti_alias;
        self.pixmap.fill_path(
            &sk_path,
            &sk_paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }

    fn stroke_path(&mut self, path: &Path, stroke: &Stroke, paint: &Paint) {
        let sk_path = path_to_skia(path);
        let color = Color::from_rgba8(
            (paint.color.r * 255.0) as u8,
            (paint.color.g * 255.0) as u8,
            (paint.color.b * 255.0) as u8,
            (paint.color.a * 255.0) as u8,
        );
        let mut sk_paint = SkPaint::default();
        sk_paint.set_color(color);
        sk_paint.anti_alias = paint.anti_alias;

        let line_cap = match stroke.line_cap {
            crate::LineCap::Butt => LineCap::Butt,
            crate::LineCap::Round => LineCap::Round,
            crate::LineCap::Square => LineCap::Square,
        };
        let line_join = match stroke.line_join {
            crate::LineJoin::Miter => LineJoin::Miter,
            crate::LineJoin::Round => LineJoin::Round,
            crate::LineJoin::Bevel => LineJoin::Bevel,
        };

        let sk_stroke = SkStroke {
            width: stroke.width,
            line_cap: line_cap,
            line_join: line_join,
            ..Default::default()
        };

        self.pixmap.stroke_path(
            &sk_path,
            &sk_paint,
            &sk_stroke,
            Transform::identity(),
            None,
        );
    }

    fn draw_glyphs(
        &mut self,
        _pos: Point,
        _glyphs: &[crate::GlyphRun],
        _paint: &Paint,
    ) {
    }

    fn draw_image(&mut self, image: &crate::Image, rect: Rect) {
        let id = self.next_image_id;
        self.next_image_id += 1;

        if let Some(mut pixmap) = Pixmap::new(image.width, image.height) {
            let w = image.width;
            let pixel_data = pixmap.data_mut();
            for y in 0..image.height {
                for x in 0..w {
                    let idx = ((y * w + x) * 4) as usize;
                    if idx + 3 < image.data.len() {
                        pixel_data[idx] = image.data[idx];
                        pixel_data[idx + 1] = image.data[idx + 1];
                        pixel_data[idx + 2] = image.data[idx + 2];
                        pixel_data[idx + 3] = image.data[idx + 3];
                    }
                }
            }
            self.image_cache.insert(id, pixmap);
        }

        if let Some(src) = self.image_cache.get(&id) {
            let transform = Transform::from_scale(
                rect.width / image.width as f32,
                rect.height / image.height as f32,
            )
            .post_translate(rect.x, rect.y);
            self.pixmap.as_mut().draw_pixmap(
                0,
                0,
                src.as_ref(),
                &tiny_skia::PixmapPaint::default(),
                transform,
                None,
            );
        }
    }

    fn draw_svg(&mut self, _svg: &crate::Svg, _rect: Rect) {}

    fn clip(&mut self, shape: &Shape) {
        let bounds = shape.path.segments.iter().fold(
            (f32::MAX, f32::MAX, f32::MIN, f32::MIN),
            |(min_x, min_y, max_x, max_y), seg| match seg {
                crate::PathSegment::MoveTo(p)
                | crate::PathSegment::LineTo(p) => (
                    min_x.min(p.x),
                    min_y.min(p.y),
                    max_x.max(p.x),
                    max_y.max(p.y),
                ),
                crate::PathSegment::QuadTo(c, p) => (
                    min_x.min(c.x).min(p.x),
                    min_y.min(c.y).min(p.y),
                    max_x.max(c.x).max(p.x),
                    max_y.max(c.y).max(p.y),
                ),
                crate::PathSegment::CubicTo(c1, c2, p) => (
                    min_x.min(c1.x).min(c2.x).min(p.x),
                    min_y.min(c1.y).min(c2.y).min(p.y),
                    max_x.max(c1.x).max(c2.x).max(p.x),
                    max_y.max(c1.y).max(c2.y).max(p.y),
                ),
                crate::PathSegment::Close => (min_x, min_y, max_x, max_y),
            },
        );
        self.clip_rect = Some(Rect {
            x: bounds.0,
            y: bounds.1,
            width: bounds.2 - bounds.0,
            height: bounds.3 - bounds.1,
        });
    }

    fn clear_clip(&mut self) {
        self.clip_rect = None;
    }

    fn push_layer(&mut self, alpha: f32, _transform: &[f32; 6]) {
        let layer_pixmap = Pixmap::new(self.width, self.height)
            .expect("Failed to create layer pixmap");
        self.layer_stack.push(LayerState {
            alpha,
            pixmap: layer_pixmap,
        });
    }

    fn pop_layer(&mut self) {
        if let Some(layer) = self.layer_stack.pop() {
            let alpha_u8 = (layer.alpha * 255.0) as u8;
            let pixel_data = self.pixmap.data_mut();
            let src_data = layer.pixmap.data();
            for y in 0..self.height {
                for x in 0..self.width {
                    let idx = (y * self.width + x) as usize * 4;
                    let sa = (src_data[idx + 3] as u16 * alpha_u8 as u16) / 255;
                    let da = 255u16 - sa;
                    pixel_data[idx] = ((src_data[idx] as u16 * sa
                        + pixel_data[idx] as u16 * da)
                        / 255) as u8;
                    pixel_data[idx + 1] = ((src_data[idx + 1] as u16 * sa
                        + pixel_data[idx + 1] as u16 * da)
                        / 255) as u8;
                    pixel_data[idx + 2] = ((src_data[idx + 2] as u16 * sa
                        + pixel_data[idx + 2] as u16 * da)
                        / 255) as u8;
                    pixel_data[idx + 3] = (sa + da).min(255) as u8;
                }
            }
        }
    }

    fn finish(&mut self) -> Frame {
        let data = self.pixmap.data().to_vec();
        Frame {
            data,
            width: self.width,
            height: self.height,
        }
    }
}

fn path_to_skia(path: &Path) -> SkPath {
    let mut pb = PathBuilder::new();
    for segment in &path.segments {
        match segment {
            crate::PathSegment::MoveTo(p) => pb.move_to(p.x, p.y),
            crate::PathSegment::LineTo(p) => pb.line_to(p.x, p.y),
            crate::PathSegment::QuadTo(c, p) => pb.quad_to(c.x, c.y, p.x, p.y),
            crate::PathSegment::CubicTo(c1, c2, p) => {
                pb.cubic_to(c1.x, c1.y, c2.x, c2.y, p.x, p.y)
            }
            crate::PathSegment::Close => pb.close(),
        }
    }
    pb.finish().unwrap_or_else(|| {
        let mut b = PathBuilder::new();
        b.move_to(0.0, 0.0);
        b.line_to(0.0, 0.0);
        b.close();
        b.finish().unwrap()
    })
}
