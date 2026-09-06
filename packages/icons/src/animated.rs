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

//! Animated icon component using montrs-motion spring physics.
//!
//! Each icon gets a physics-based animation profile on hover.
//! Requires the `animated` feature flag.

use crate::{glyph::Glyph, icon::DEFAULT_SIZE};
use leptos::{prelude::*, text_prop::TextProp};
use montrs_motion::FrameLoop;

/// Animation profile for a glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationProfile {
    PathDraw,
    Pulse,
    Shake,
    Spin,
    Nod,
    Bounce,
    Ping,
    None,
}

/// Animated icon component with spring physics on hover.
///
/// Sizing: pass Tailwind size utilities via `class` (e.g. `class="w-6 h-6"`)
/// or the `size` prop (e.g. `size="24"`). The class is applied directly to
/// the `<svg>` element so CSS width/height override the presentation
/// attributes, mirroring the behavior of [`crate::icon::Icon`].
///
/// Every icon animates on hover: the auto-detected profile maps to a
/// frame-loop (spin/shake/nod), a CSS keyframe class (pulse/bounce/ping), or
/// a real-length stroke draw (default), so no glyph is ever left static.
#[component]
pub fn AnimatedIcon(
    #[prop(into)] glyph: Signal<Glyph>,
    #[prop(into, optional)] class: Option<TextProp>,
    #[prop(into, optional)] size: Option<TextProp>,
    #[prop(into, optional)] fill: Option<TextProp>,
    #[prop(into, optional)] stroke: Option<TextProp>,
    #[prop(into, optional)] stroke_width: Option<TextProp>,
    /// Override the auto-detected animation profile (`None` = auto).
    #[prop(into, optional)]
    profile: Signal<Option<AnimationProfile>>,
) -> impl IntoView {
    let profile = Memo::new(move |_| {
        profile
            .get()
            .unwrap_or_else(|| animation_profile(glyph.get()))
    });

    let svg_data = TextProp::from(move || glyph.get().svg());
    let class_val = class.unwrap_or_else(|| "".into());
    let size_val = size.unwrap_or_else(|| DEFAULT_SIZE.into());
    let size2 = size_val.clone();
    let fill_val = fill.unwrap_or_else(|| "none".into());
    let stroke_val = stroke.unwrap_or_else(|| "currentColor".into());
    let sw = stroke_width.unwrap_or_else(|| "1.5".into());

    // Spring animation values
    let scale = RwSignal::new(1.0);
    let rotate = RwSignal::new(0.0);
    let translate_y = RwSignal::new(0.0);
    let is_spinning = RwSignal::new(false);
    // CSS keyframe class applied on hover (pulse / bounce / ping).
    let css_class = RwSignal::new("");

    let on_enter = move |ev: leptos::ev::MouseEvent| {
        match profile.get() {
            AnimationProfile::Pulse => css_class.set("montrs-pulse"),
            AnimationProfile::Bounce => css_class.set("montrs-bounce"),
            AnimationProfile::Ping => css_class.set("montrs-ping"),
            AnimationProfile::Spin => {
                is_spinning.set(true);
                let start = FrameLoop::now();
                FrameLoop::on_frame(move || {
                    if !is_spinning.get() {
                        return false;
                    }
                    let elapsed = FrameLoop::now() - start;
                    rotate.set((elapsed * 360.0 * 1.5) % 360.0);
                    true
                });
            }
            AnimationProfile::Shake => {
                is_spinning.set(true);
                let start = FrameLoop::now();
                FrameLoop::on_frame(move || {
                    let elapsed = FrameLoop::now() - start;
                    if elapsed > 0.6 || !is_spinning.get() {
                        return false;
                    }
                    rotate.set((elapsed * 40.0).sin() * 10.0);
                    true
                });
            }
            AnimationProfile::Nod => {
                translate_y.set(-4.0);
                FrameLoop::on_frame(move || {
                    let current: f64 = translate_y.get();
                    let next: f64 = current + (0.0 - current) * 0.2;
                    translate_y.set(next);
                    next.abs() > 0.1
                });
            }
            AnimationProfile::PathDraw => {
                if let Some(svg) = resolve_svg(&ev) {
                    draw_svg(&svg, 350, 0);
                }
            }
            AnimationProfile::None => {}
        }
    };

    let on_leave = move |ev: leptos::ev::MouseEvent| {
        css_class.set("");
        is_spinning.set(false);
        scale.set(1.0);
        rotate.set(0.0);
        translate_y.set(0.0);
        if profile.get() == AnimationProfile::PathDraw
            && let Some(svg) = resolve_svg(&ev)
        {
            reset_draw(&svg);
        }
    };

    // Per-frame transform lives on the `<svg>`; CSS keyframe classes
    // (pulse/bounce/ping) are layered on top via the class attribute.
    let svg_style = move || {
        let mut styles = format!(
            "transform: scale({}) rotate({})deg translateY({}px); \
             transform-origin: center;",
            scale.get(),
            rotate.get(),
            translate_y.get()
        );
        if is_spinning.get() {
            styles.push_str(" transition: none;");
        } else {
            styles.push_str(
                " transition: transform 0.35s cubic-bezier(0.16,1,0.3,1);",
            );
        }
        styles
    };

    view! {
        <span
            class="inline-flex cursor-pointer"
            on:mouseenter=on_enter
            on:mouseleave=on_leave
        >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class=move || {
                  let extra = css_class.get();
                  if extra.is_empty() {
                      class_val.get().to_string()
                  } else {
                      format!("{} {}", class_val.get(), extra)
                  }
              }
              width=move || size_val.get()
              height=move || size2.get()
              viewBox="0 0 24 24"
              fill=move || fill_val.get()
              stroke=move || stroke_val.get()
              stroke-width=move || sw.get()
              stroke-linecap="round"
              stroke-linejoin="round"
              style=svg_style
              inner_html=move || svg_data.get()
            />
        </span>
    }
}

/// Resolve the `<svg>` element from a mouse event by walking up the DOM from
/// the event target (avoids needing a typed NodeRef for the SVG namespace).
#[allow(unused_variables)]
fn resolve_svg(ev: &leptos::ev::MouseEvent) -> Option<web_sys::SvgElement> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        let mut node = ev.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok());
        while let Some(n) = node.clone() {
            if let Ok(el) = n.clone().dyn_into::<web_sys::Element>() {
                if el.tag_name().eq_ignore_ascii_case("svg") {
                    return el.dyn_into::<web_sys::SvgElement>().ok();
                }
            }
            node = n.parent_element().map(|e| e.into());
        }
    }
    let _ = ev;
    None
}

/// Stroke-draw animation using the browser-measured path lengths (lepticons'
/// approach): frame 1 hides every geometry element by setting its own
/// `stroke-dasharray`/`dashoffset`, frame 2 enables the transition and draws
/// to 0. Works on any stroke-based glyph, so no icon is left unanimated.
#[allow(unused_variables)]
fn draw_svg(svg: &web_sys::SvgElement, duration_ms: u32, delay_ms: u32) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::Closure;
        use wasm_bindgen::JsCast;

        let svg: web_sys::Element = svg.clone().into();
        // Apply per-element measured dash lengths (hides every stroke).
        apply_measured_lengths(&svg);

        // Frame 1: also mark them static.
        let svg_frame1 = svg.clone();
        let closure1 = Closure::wrap(Box::new(move || {
            set_geometry_state(&svg_frame1, |style| {
                let _ = style.set_property("transition", "none");
            });
            // Frame 2: enable the transition and draw to visible.
            if let Some(win) = web_sys::window() {
                let svg_frame2 = svg_frame1.clone();
                let closure2 = Closure::wrap(Box::new(move || {
                    set_geometry_state(&svg_frame2, |style| {
                        let _ = style.set_property(
                            "transition",
                            &format!(
                                "stroke-dashoffset {duration_ms}ms ease-in-out \
                                 {delay_ms}ms"
                            ),
                        );
                        let _ = style.set_property("stroke-dashoffset", "0");
                    });
                }) as Box<dyn FnMut()>);
                let _ = win.request_animation_frame(closure2.as_ref().unchecked_ref());
                closure2.forget();
            }
        }) as Box<dyn FnMut()>);
        if let Some(win) = web_sys::window() {
            let _ = win.request_animation_frame(closure1.as_ref().unchecked_ref());
        }
        closure1.forget();
    }
}

#[cfg(target_arch = "wasm32")]
fn set_geometry_state<F: Fn(&web_sys::CssStyleDeclaration)>(svg: &web_sys::Element, f: F) {
    use wasm_bindgen::JsCast;
    let children = svg.children();
    for i in 0..children.length() {
        let Some(child) = children.item(i) else {
            continue;
        };
        if child.dyn_ref::<web_sys::SvgGeometryElement>().is_none() {
            continue;
        }
        let child: web_sys::SvgElement = child.unchecked_into();
        f(&child.style());
    }
}

#[cfg(target_arch = "wasm32")]
fn apply_measured_lengths(svg: &web_sys::Element) {
    use wasm_bindgen::JsCast;
    let children = svg.children();
    for i in 0..children.length() {
        let Some(child) = children.item(i) else {
            continue;
        };
        if child.dyn_ref::<web_sys::SvgGeometryElement>().is_none() {
            continue;
        }
        let geom: web_sys::SvgGeometryElement = child.clone().unchecked_into();
        let len: f32 = geom.get_total_length();
        if len <= 0.0 {
            continue;
        }
        let svg_el: web_sys::SvgElement = child.unchecked_into();
        let style = svg_el.style();
        let _ = style.set_property("stroke-dasharray", &len.to_string());
        let _ = style.set_property("stroke-dashoffset", &len.to_string());
    }
}

/// Restore default stroke styling after a draw animation.
#[allow(unused_variables)]
fn reset_draw(svg: &web_sys::SvgElement) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        let svg: web_sys::Element = svg.clone().into();
        let children = svg.children();
        for i in 0..children.length() {
            if let Some(child) = children.item(i) {
                if child.dyn_ref::<web_sys::SvgGeometryElement>().is_some() {
                    let child: web_sys::SvgElement = child.unchecked_into();
                    let style = child.style();
                    let _ = style.remove_property("stroke-dasharray");
                    let _ = style.remove_property("stroke-dashoffset");
                    let _ = style.remove_property("transition");
                }
            }
        }
    }
}

/// The auto-detected animation profile for a glyph.
pub fn animation_profile(glyph: Glyph) -> AnimationProfile {
    let name = glyph.name();
    if name.contains("Loader")
        || name.contains("Spinner")
        || name.contains("Cog")
        || name.contains("Refresh")
        || name.contains("Sync")
        || name.contains("Rotate")
        || name == "LoaderCircle"
    {
        return AnimationProfile::Spin;
    }
    if name.contains("Heart")
        || name.contains("Thumbs")
        || name.contains("Activity")
        || name.contains("Wifi")
        || name.contains("Signal")
        || name.contains("Radio")
        || name.contains("Rss")
        || name.contains("Antenna")
    {
        return AnimationProfile::Pulse;
    }
    if name.contains("Bell")
        || name.contains("Alert")
        || name.contains("Alarm")
        || name.contains("Notification")
        || name == "Moon"
    {
        return AnimationProfile::Shake;
    }
    if name.contains("Arrow")
        || name.contains("Chevron")
        || name.contains("Move")
        || name.contains("Mouse")
        || name.contains("Hand")
    {
        return AnimationProfile::Bounce;
    }
    if name.contains("Radar")
        || name.contains("MapPin")
        || name.contains("Anchor")
        || name.contains("Target")
        || name.contains("Locate")
    {
        return AnimationProfile::Ping;
    }
    if name.contains("Search")
        || name.contains("Navigation")
        || name.contains("Compass")
        || name.contains("Crosshair")
    {
        return AnimationProfile::Nod;
    }
    AnimationProfile::PathDraw
}

pub mod animated_registry {
    use super::AnimatedIcon;
    use crate::glyph::Glyph;
    use leptos::prelude::*;

    macro_rules! def_animated_icon {
        ($name:ident) => {
            #[doc(hidden)]
            #[component]
            pub fn $name(
                #[prop(into, optional)] class: String,
            ) -> impl IntoView {
                view! { <AnimatedIcon glyph=Glyph::$name class=class /> }
            }
        };
    }

    def_animated_icon!(AArrowDown);
    def_animated_icon!(AArrowUp);
    def_animated_icon!(ALargeSmall);
    def_animated_icon!(Accessibility);
    def_animated_icon!(Activity);
    def_animated_icon!(Ad);
    def_animated_icon!(AirVent);
    def_animated_icon!(Airplay);
    def_animated_icon!(AlarmClockCheck);
    def_animated_icon!(AlarmClockMinus);
    def_animated_icon!(AlarmClockOff);
    def_animated_icon!(AlarmClockPlus);
    def_animated_icon!(AlarmClock);
    def_animated_icon!(AlarmSmoke);
    def_animated_icon!(Album);
    def_animated_icon!(AlignCenterHorizontal);
    def_animated_icon!(AlignCenterVertical);
    def_animated_icon!(AlignEndHorizontal);
    def_animated_icon!(AlignEndVertical);
    def_animated_icon!(AlignHorizontalDistributeCenter);
    def_animated_icon!(AlignHorizontalDistributeEnd);
    def_animated_icon!(AlignHorizontalDistributeStart);
    def_animated_icon!(AlignHorizontalJustifyCenter);
    def_animated_icon!(AlignHorizontalJustifyEnd);
    def_animated_icon!(AlignHorizontalJustifyStart);
    def_animated_icon!(AlignHorizontalSpaceAround);
    def_animated_icon!(AlignHorizontalSpaceBetween);
    def_animated_icon!(AlignStartHorizontal);
    def_animated_icon!(AlignStartVertical);
    def_animated_icon!(AlignVerticalDistributeCenter);
    def_animated_icon!(AlignVerticalDistributeEnd);
    def_animated_icon!(AlignVerticalDistributeStart);
    def_animated_icon!(AlignVerticalJustifyCenter);
    def_animated_icon!(AlignVerticalJustifyEnd);
    def_animated_icon!(AlignVerticalJustifyStart);
    def_animated_icon!(AlignVerticalSpaceAround);
    def_animated_icon!(AlignVerticalSpaceBetween);
    def_animated_icon!(Ambulance);
    def_animated_icon!(Ampersand);
    def_animated_icon!(Ampersands);
    def_animated_icon!(Amphora);
    def_animated_icon!(Anchor);
    def_animated_icon!(Angry);
    def_animated_icon!(Annoyed);
    def_animated_icon!(Antenna);
    def_animated_icon!(Anvil);
    def_animated_icon!(Aperture);
    def_animated_icon!(AppWindowMac);
    def_animated_icon!(AppWindow);
    def_animated_icon!(Apple);
    def_animated_icon!(ArchiveRestore);
    def_animated_icon!(ArchiveX);
    def_animated_icon!(Archive);
    def_animated_icon!(Armchair);
    def_animated_icon!(ArrowBigDownDash);
    def_animated_icon!(ArrowBigDown);
    def_animated_icon!(ArrowBigLeftDash);
    def_animated_icon!(ArrowBigLeft);
    def_animated_icon!(ArrowBigRightDash);
    def_animated_icon!(ArrowBigRight);
    def_animated_icon!(ArrowBigUpDash);
    def_animated_icon!(ArrowBigUp);
    def_animated_icon!(ArrowDown01);
    def_animated_icon!(ArrowDown10);
    def_animated_icon!(ArrowDownAZ);
    def_animated_icon!(ArrowDownFromLine);
    def_animated_icon!(ArrowDownLeft);
    def_animated_icon!(ArrowDownNarrowWide);
    def_animated_icon!(ArrowDownRight);
    def_animated_icon!(ArrowDownToDot);
    def_animated_icon!(ArrowDownToLine);
    def_animated_icon!(ArrowDownUp);
    def_animated_icon!(ArrowDownWideNarrow);
    def_animated_icon!(ArrowDownZA);
    def_animated_icon!(ArrowDown);
    def_animated_icon!(ArrowLeftFromLine);
    def_animated_icon!(ArrowLeftRight);
    def_animated_icon!(ArrowLeftToLine);
    def_animated_icon!(ArrowLeft);
    def_animated_icon!(ArrowRightFromLine);
    def_animated_icon!(ArrowRightLeft);
    def_animated_icon!(ArrowRightToLine);
    def_animated_icon!(ArrowRight);
    def_animated_icon!(ArrowUp01);
    def_animated_icon!(ArrowUp10);
    def_animated_icon!(ArrowUpAZ);
    def_animated_icon!(ArrowUpDown);
    def_animated_icon!(ArrowUpFromDot);
    def_animated_icon!(ArrowUpFromLine);
    def_animated_icon!(ArrowUpLeft);
    def_animated_icon!(ArrowUpNarrowWide);
    def_animated_icon!(ArrowUpRight);
    def_animated_icon!(ArrowUpToLine);
    def_animated_icon!(ArrowUpWideNarrow);
    def_animated_icon!(ArrowUpZA);
    def_animated_icon!(ArrowUp);
    def_animated_icon!(ArrowsUpFromLine);
    def_animated_icon!(Asterisk);
    def_animated_icon!(Astroid);
    def_animated_icon!(AtSign);
    def_animated_icon!(Atom);
    def_animated_icon!(AudioLines);
    def_animated_icon!(AudioWaveform);
    def_animated_icon!(Award);
    def_animated_icon!(Axe);
    def_animated_icon!(Axis3D);
    def_animated_icon!(Baby);
    def_animated_icon!(Backpack);
    def_animated_icon!(BadgeAlert);
    def_animated_icon!(BadgeCent);
    def_animated_icon!(BadgeCheck);
    def_animated_icon!(BadgeDollarSign);
    def_animated_icon!(BadgeEuro);
    def_animated_icon!(BadgeIndianRupee);
    def_animated_icon!(BadgeInfo);
    def_animated_icon!(BadgeJapaneseYen);
    def_animated_icon!(BadgeMinus);
    def_animated_icon!(BadgePercent);
    def_animated_icon!(BadgePlus);
    def_animated_icon!(BadgePoundSterling);
    def_animated_icon!(BadgeQuestionMark);
    def_animated_icon!(BadgeRussianRuble);
    def_animated_icon!(BadgeSwissFranc);
    def_animated_icon!(BadgeTurkishLira);
    def_animated_icon!(BadgeX);
    def_animated_icon!(Badge);
    def_animated_icon!(BaggageClaim);
    def_animated_icon!(Balloon);
    def_animated_icon!(Ban);
    def_animated_icon!(Banana);
    def_animated_icon!(Bandage);
    def_animated_icon!(BanknoteArrowDown);
    def_animated_icon!(BanknoteArrowUp);
    def_animated_icon!(BanknoteCheck);
    def_animated_icon!(BanknoteX);
    def_animated_icon!(Banknote);
    def_animated_icon!(Barcode);
    def_animated_icon!(Barrel);
    def_animated_icon!(Baseline);
    def_animated_icon!(Bath);
    def_animated_icon!(BatteryCharging);
    def_animated_icon!(BatteryFull);
    def_animated_icon!(BatteryLow);
    def_animated_icon!(BatteryMedium);
    def_animated_icon!(BatteryPlus);
    def_animated_icon!(BatteryWarning);
    def_animated_icon!(Battery);
    def_animated_icon!(Beaker);
    def_animated_icon!(BeanOff);
    def_animated_icon!(Bean);
    def_animated_icon!(BedDouble);
    def_animated_icon!(BedSingle);
    def_animated_icon!(Bed);
    def_animated_icon!(BeefOff);
    def_animated_icon!(Beef);
    def_animated_icon!(BeerOff);
    def_animated_icon!(Beer);
    def_animated_icon!(BellCheck);
    def_animated_icon!(BellDot);
    def_animated_icon!(BellElectric);
    def_animated_icon!(BellMinus);
    def_animated_icon!(BellOff);
    def_animated_icon!(BellPlus);
    def_animated_icon!(BellRing);
    def_animated_icon!(Bell);
    def_animated_icon!(BetweenHorizontalEnd);
    def_animated_icon!(BetweenHorizontalStart);
    def_animated_icon!(BetweenVerticalEnd);
    def_animated_icon!(BetweenVerticalStart);
    def_animated_icon!(BicepsFlexed);
    def_animated_icon!(Bike);
    def_animated_icon!(Binary);
    def_animated_icon!(Binoculars);
    def_animated_icon!(Biohazard);
    def_animated_icon!(Bird);
    def_animated_icon!(Birdhouse);
    def_animated_icon!(Bitcoin);
    def_animated_icon!(Blend);
    def_animated_icon!(Blender);
    def_animated_icon!(Blinds);
    def_animated_icon!(Blocks);
    def_animated_icon!(BluetoothConnected);
    def_animated_icon!(BluetoothOff);
    def_animated_icon!(BluetoothSearching);
    def_animated_icon!(Bluetooth);
    def_animated_icon!(Bold);
    def_animated_icon!(Bolt);
    def_animated_icon!(Bomb);
    def_animated_icon!(BoneFracture);
    def_animated_icon!(Bone);
    def_animated_icon!(BookA);
    def_animated_icon!(BookAlert);
    def_animated_icon!(BookAudio);
    def_animated_icon!(BookCheck);
    def_animated_icon!(BookCopy);
    def_animated_icon!(BookDashed);
    def_animated_icon!(BookDown);
    def_animated_icon!(BookHeadphones);
    def_animated_icon!(BookHeart);
    def_animated_icon!(BookImage);
    def_animated_icon!(BookKey);
    def_animated_icon!(BookLock);
    def_animated_icon!(BookMarked);
    def_animated_icon!(BookMinus);
    def_animated_icon!(BookOpenCheck);
    def_animated_icon!(BookOpenText);
    def_animated_icon!(BookOpen);
    def_animated_icon!(BookPlus);
    def_animated_icon!(BookSearch);
    def_animated_icon!(BookText);
    def_animated_icon!(BookType);
    def_animated_icon!(BookUp2);
    def_animated_icon!(BookUp);
    def_animated_icon!(BookUser);
    def_animated_icon!(BookX);
    def_animated_icon!(Book);
    def_animated_icon!(BookmarkCheck);
    def_animated_icon!(BookmarkMinus);
    def_animated_icon!(BookmarkOff);
    def_animated_icon!(BookmarkPlus);
    def_animated_icon!(BookmarkX);
    def_animated_icon!(Bookmark);
    def_animated_icon!(BoomBox);
    def_animated_icon!(BotMessageSquare);
    def_animated_icon!(BotOff);
    def_animated_icon!(Bot);
    def_animated_icon!(BottleWine);
    def_animated_icon!(BowArrow);
    def_animated_icon!(Box);
    def_animated_icon!(Boxes);
    def_animated_icon!(Braces);
    def_animated_icon!(Brackets);
    def_animated_icon!(BrainCircuit);
    def_animated_icon!(BrainCog);
    def_animated_icon!(Brain);
    def_animated_icon!(BrickWallFire);
    def_animated_icon!(BrickWallShield);
    def_animated_icon!(BrickWall);
    def_animated_icon!(BriefcaseBusiness);
    def_animated_icon!(BriefcaseConveyorBelt);
    def_animated_icon!(BriefcaseMedical);
    def_animated_icon!(Briefcase);
    def_animated_icon!(BringToFront);
    def_animated_icon!(Broccoli);
    def_animated_icon!(BrushCleaning);
    def_animated_icon!(Brush);
    def_animated_icon!(Bubbles);
    def_animated_icon!(BugOff);
    def_animated_icon!(BugPlay);
    def_animated_icon!(Bug);
    def_animated_icon!(Building2);
    def_animated_icon!(Building);
    def_animated_icon!(BusFront);
    def_animated_icon!(Bus);
    def_animated_icon!(CableCar);
    def_animated_icon!(Cable);
    def_animated_icon!(CakeSlice);
    def_animated_icon!(Cake);
    def_animated_icon!(Calculator);
    def_animated_icon!(Calendar1);
    def_animated_icon!(CalendarArrowDown);
    def_animated_icon!(CalendarArrowUp);
    def_animated_icon!(CalendarCheck2);
    def_animated_icon!(CalendarCheck);
    def_animated_icon!(CalendarClock);
    def_animated_icon!(CalendarCog);
    def_animated_icon!(CalendarDays);
    def_animated_icon!(CalendarFold);
    def_animated_icon!(CalendarHeart);
    def_animated_icon!(CalendarMinus2);
    def_animated_icon!(CalendarMinus);
    def_animated_icon!(CalendarOff);
    def_animated_icon!(CalendarPlus2);
    def_animated_icon!(CalendarPlus);
    def_animated_icon!(CalendarRange);
    def_animated_icon!(CalendarSearch);
    def_animated_icon!(CalendarSync);
    def_animated_icon!(CalendarX2);
    def_animated_icon!(CalendarX);
    def_animated_icon!(Calendar);
    def_animated_icon!(Calendars);
    def_animated_icon!(CameraOff);
    def_animated_icon!(Camera);
    def_animated_icon!(CandyCane);
    def_animated_icon!(CandyOff);
    def_animated_icon!(Candy);
    def_animated_icon!(CannabisOff);
    def_animated_icon!(Cannabis);
    def_animated_icon!(CaptionsOff);
    def_animated_icon!(Captions);
    def_animated_icon!(CarFront);
    def_animated_icon!(CarTaxiFront);
    def_animated_icon!(Car);
    def_animated_icon!(Caravan);
    def_animated_icon!(CardSim);
    def_animated_icon!(Carrot);
    def_animated_icon!(CaseLower);
    def_animated_icon!(CaseSensitive);
    def_animated_icon!(CaseUpper);
    def_animated_icon!(CassetteTape);
    def_animated_icon!(Cast);
    def_animated_icon!(Castle);
    def_animated_icon!(Cat);
    def_animated_icon!(CctvOff);
    def_animated_icon!(Cctv);
    def_animated_icon!(ChartArea);
    def_animated_icon!(ChartBarBig);
    def_animated_icon!(ChartBarDecreasing);
    def_animated_icon!(ChartBarIncreasing);
    def_animated_icon!(ChartBarStacked);
    def_animated_icon!(ChartBar);
    def_animated_icon!(ChartCandlestick);
    def_animated_icon!(ChartColumnBig);
    def_animated_icon!(ChartColumnDecreasing);
    def_animated_icon!(ChartColumnIncreasing);
    def_animated_icon!(ChartColumnStacked);
    def_animated_icon!(ChartColumn);
    def_animated_icon!(ChartGantt);
    def_animated_icon!(ChartLine);
    def_animated_icon!(ChartNetwork);
    def_animated_icon!(ChartNoAxesColumnDecreasing);
    def_animated_icon!(ChartNoAxesColumnIncreasing);
    def_animated_icon!(ChartNoAxesColumn);
    def_animated_icon!(ChartNoAxesCombined);
    def_animated_icon!(ChartNoAxesGantt);
    def_animated_icon!(ChartPie);
    def_animated_icon!(ChartScatter);
    def_animated_icon!(ChartSpline);
    def_animated_icon!(CheckCheck);
    def_animated_icon!(CheckLine);
    def_animated_icon!(Check);
    def_animated_icon!(ChefHat);
    def_animated_icon!(Cherry);
    def_animated_icon!(ChessBishop);
    def_animated_icon!(ChessKing);
    def_animated_icon!(ChessKnight);
    def_animated_icon!(ChessPawn);
    def_animated_icon!(ChessQueen);
    def_animated_icon!(ChessRook);
    def_animated_icon!(ChevronDown);
    def_animated_icon!(ChevronFirst);
    def_animated_icon!(ChevronLast);
    def_animated_icon!(ChevronLeft);
    def_animated_icon!(ChevronRight);
    def_animated_icon!(ChevronUp);
    def_animated_icon!(ChevronsDownUp);
    def_animated_icon!(ChevronsDown);
    def_animated_icon!(ChevronsLeftRightEllipsis);
    def_animated_icon!(ChevronsLeftRight);
    def_animated_icon!(ChevronsLeft);
    def_animated_icon!(ChevronsRightLeft);
    def_animated_icon!(ChevronsRight);
    def_animated_icon!(ChevronsUpDown);
    def_animated_icon!(ChevronsUp);
    def_animated_icon!(Church);
    def_animated_icon!(CigaretteOff);
    def_animated_icon!(Cigarette);
    def_animated_icon!(CircleAlert);
    def_animated_icon!(CircleArrowDown);
    def_animated_icon!(CircleArrowLeft);
    def_animated_icon!(CircleArrowOutDownLeft);
    def_animated_icon!(CircleArrowOutDownRight);
    def_animated_icon!(CircleArrowOutUpLeft);
    def_animated_icon!(CircleArrowOutUpRight);
    def_animated_icon!(CircleArrowRight);
    def_animated_icon!(CircleArrowUp);
    def_animated_icon!(CircleCheckBig);
    def_animated_icon!(CircleCheck);
    def_animated_icon!(CircleChevronDown);
    def_animated_icon!(CircleChevronLeft);
    def_animated_icon!(CircleChevronRight);
    def_animated_icon!(CircleChevronUp);
    def_animated_icon!(CircleDashed);
    def_animated_icon!(CircleDivide);
    def_animated_icon!(CircleDollarSign);
    def_animated_icon!(CircleDotDashed);
    def_animated_icon!(CircleDot);
    def_animated_icon!(CircleEllipsis);
    def_animated_icon!(CircleEqual);
    def_animated_icon!(CircleFadingArrowUp);
    def_animated_icon!(CircleFadingPlus);
    def_animated_icon!(CircleGauge);
    def_animated_icon!(CircleMinus);
    def_animated_icon!(CircleOff);
    def_animated_icon!(CircleParkingOff);
    def_animated_icon!(CircleParking);
    def_animated_icon!(CirclePause);
    def_animated_icon!(CirclePercent);
    def_animated_icon!(CirclePile);
    def_animated_icon!(CirclePlay);
    def_animated_icon!(CirclePlus);
    def_animated_icon!(CirclePoundSterling);
    def_animated_icon!(CirclePower);
    def_animated_icon!(CircleQuestionMark);
    def_animated_icon!(CircleSlash2);
    def_animated_icon!(CircleSlash);
    def_animated_icon!(CircleSmall);
    def_animated_icon!(CircleStar);
    def_animated_icon!(CircleStop);
    def_animated_icon!(CircleUserRound);
    def_animated_icon!(CircleUser);
    def_animated_icon!(CircleX);
    def_animated_icon!(Circle);
    def_animated_icon!(CircuitBoard);
    def_animated_icon!(Citrus);
    def_animated_icon!(Clapperboard);
    def_animated_icon!(ClipboardCheck);
    def_animated_icon!(ClipboardClock);
    def_animated_icon!(ClipboardCopy);
    def_animated_icon!(ClipboardList);
    def_animated_icon!(ClipboardMinus);
    def_animated_icon!(ClipboardPaste);
    def_animated_icon!(ClipboardPenLine);
    def_animated_icon!(ClipboardPen);
    def_animated_icon!(ClipboardPlus);
    def_animated_icon!(ClipboardType);
    def_animated_icon!(ClipboardX);
    def_animated_icon!(Clipboard);
    def_animated_icon!(Clock1);
    def_animated_icon!(Clock10);
    def_animated_icon!(Clock11);
    def_animated_icon!(Clock12);
    def_animated_icon!(Clock2);
    def_animated_icon!(Clock3);
    def_animated_icon!(Clock4);
    def_animated_icon!(Clock5);
    def_animated_icon!(Clock6);
    def_animated_icon!(Clock7);
    def_animated_icon!(Clock8);
    def_animated_icon!(Clock9);
    def_animated_icon!(ClockAlert);
    def_animated_icon!(ClockArrowDown);
    def_animated_icon!(ClockArrowLeft);
    def_animated_icon!(ClockArrowRight);
    def_animated_icon!(ClockArrowUp);
    def_animated_icon!(ClockCheck);
    def_animated_icon!(ClockFading);
    def_animated_icon!(ClockPlus);
    def_animated_icon!(Clock);
    def_animated_icon!(ClosedCaption);
    def_animated_icon!(CloudAlert);
    def_animated_icon!(CloudBackup);
    def_animated_icon!(CloudCheck);
    def_animated_icon!(CloudCog);
    def_animated_icon!(CloudDownload);
    def_animated_icon!(CloudDrizzle);
    def_animated_icon!(CloudFog);
    def_animated_icon!(CloudHail);
    def_animated_icon!(CloudLightning);
    def_animated_icon!(CloudMoonRain);
    def_animated_icon!(CloudMoon);
    def_animated_icon!(CloudOff);
    def_animated_icon!(CloudRainWind);
    def_animated_icon!(CloudRain);
    def_animated_icon!(CloudSnow);
    def_animated_icon!(CloudSunRain);
    def_animated_icon!(CloudSun);
    def_animated_icon!(CloudSync);
    def_animated_icon!(CloudUpload);
    def_animated_icon!(Cloud);
    def_animated_icon!(Cloudy);
    def_animated_icon!(Clover);
    def_animated_icon!(Club);
    def_animated_icon!(CodeXml);
    def_animated_icon!(Code);
    def_animated_icon!(Coffee);
    def_animated_icon!(Cog);
    def_animated_icon!(Coins);
    def_animated_icon!(Columns2);
    def_animated_icon!(Columns3Cog);
    def_animated_icon!(Columns3);
    def_animated_icon!(Columns4);
    def_animated_icon!(Combine);
    def_animated_icon!(Command);
    def_animated_icon!(Compass);
    def_animated_icon!(Component);
    def_animated_icon!(Computer);
    def_animated_icon!(ConciergeBell);
    def_animated_icon!(Cone);
    def_animated_icon!(Construction);
    def_animated_icon!(ContactRound);
    def_animated_icon!(Contact);
    def_animated_icon!(Container);
    def_animated_icon!(Contrast);
    def_animated_icon!(Cookie);
    def_animated_icon!(CookingPot);
    def_animated_icon!(CopyCheck);
    def_animated_icon!(CopyMinus);
    def_animated_icon!(CopyPlus);
    def_animated_icon!(CopySlash);
    def_animated_icon!(CopyX);
    def_animated_icon!(Copy);
    def_animated_icon!(Copyleft);
    def_animated_icon!(Copyright);
    def_animated_icon!(CornerDownLeft);
    def_animated_icon!(CornerDownRight);
    def_animated_icon!(CornerLeftDown);
    def_animated_icon!(CornerLeftUp);
    def_animated_icon!(CornerRightDown);
    def_animated_icon!(CornerRightUp);
    def_animated_icon!(CornerUpLeft);
    def_animated_icon!(CornerUpRight);
    def_animated_icon!(Cpu);
    def_animated_icon!(CreativeCommons);
    def_animated_icon!(CreditCard);
    def_animated_icon!(Croissant);
    def_animated_icon!(Crop);
    def_animated_icon!(Cross);
    def_animated_icon!(Crosshair);
    def_animated_icon!(Crown);
    def_animated_icon!(Cuboid);
    def_animated_icon!(CupSoda);
    def_animated_icon!(Currency);
    def_animated_icon!(Cylinder);
    def_animated_icon!(Dam);
    def_animated_icon!(DatabaseArrowDown);
    def_animated_icon!(DatabaseArrowUp);
    def_animated_icon!(DatabaseBackup);
    def_animated_icon!(DatabaseCheck);
    def_animated_icon!(DatabaseMinus);
    def_animated_icon!(DatabasePlus);
    def_animated_icon!(DatabaseSearch);
    def_animated_icon!(DatabaseX);
    def_animated_icon!(DatabaseZap);
    def_animated_icon!(Database);
    def_animated_icon!(DecimalsArrowLeft);
    def_animated_icon!(DecimalsArrowRight);
    def_animated_icon!(Delete);
    def_animated_icon!(Dessert);
    def_animated_icon!(Diameter);
    def_animated_icon!(DiamondMinus);
    def_animated_icon!(DiamondPercent);
    def_animated_icon!(DiamondPlus);
    def_animated_icon!(Diamond);
    def_animated_icon!(Dice1);
    def_animated_icon!(Dice2);
    def_animated_icon!(Dice3);
    def_animated_icon!(Dice4);
    def_animated_icon!(Dice5);
    def_animated_icon!(Dice6);
    def_animated_icon!(Dices);
    def_animated_icon!(Diff);
    def_animated_icon!(Disc2);
    def_animated_icon!(Disc3);
    def_animated_icon!(DiscAlbum);
    def_animated_icon!(Disc);
    def_animated_icon!(Divide);
    def_animated_icon!(DnaOff);
    def_animated_icon!(Dna);
    def_animated_icon!(Dock);
    def_animated_icon!(Dog);
    def_animated_icon!(DollarSign);
    def_animated_icon!(Donut);
    def_animated_icon!(DoorClosedLocked);
    def_animated_icon!(DoorClosed);
    def_animated_icon!(DoorOpen);
    def_animated_icon!(Dot);
    def_animated_icon!(Download);
    def_animated_icon!(DraftingCompass);
    def_animated_icon!(Drama);
    def_animated_icon!(Drill);
    def_animated_icon!(Drone);
    def_animated_icon!(DropletOff);
    def_animated_icon!(Droplet);
    def_animated_icon!(Droplets);
    def_animated_icon!(Drum);
    def_animated_icon!(Drumstick);
    def_animated_icon!(Dumbbell);
    def_animated_icon!(EarOff);
    def_animated_icon!(Ear);
    def_animated_icon!(EarthLock);
    def_animated_icon!(Earth);
    def_animated_icon!(Eclipse);
    def_animated_icon!(EggFried);
    def_animated_icon!(EggOff);
    def_animated_icon!(Egg);
    def_animated_icon!(Ellipse);
    def_animated_icon!(EllipsisVertical);
    def_animated_icon!(Ellipsis);
    def_animated_icon!(EqualApproximately);
    def_animated_icon!(EqualNot);
    def_animated_icon!(Equal);
    def_animated_icon!(Eraser);
    def_animated_icon!(EthernetPort);
    def_animated_icon!(Euro);
    def_animated_icon!(EvCharger);
    def_animated_icon!(Expand);
    def_animated_icon!(ExternalLink);
    def_animated_icon!(EyeClosed);
    def_animated_icon!(EyeDashed);
    def_animated_icon!(EyeOff);
    def_animated_icon!(Eye);
    def_animated_icon!(Factory);
    def_animated_icon!(Fan);
    def_animated_icon!(FastForward);
    def_animated_icon!(Feather);
    def_animated_icon!(Fence);
    def_animated_icon!(FerrisWheel);
    def_animated_icon!(FileArchive);
    def_animated_icon!(FileAxis3D);
    def_animated_icon!(FileBadge);
    def_animated_icon!(FileBox);
    def_animated_icon!(FileBracesCorner);
    def_animated_icon!(FileBraces);
    def_animated_icon!(FileChartColumnIncreasing);
    def_animated_icon!(FileChartColumn);
    def_animated_icon!(FileChartLine);
    def_animated_icon!(FileChartPie);
    def_animated_icon!(FileCheckCorner);
    def_animated_icon!(FileCheck);
    def_animated_icon!(FileClock);
    def_animated_icon!(FileCodeCorner);
    def_animated_icon!(FileCode);
    def_animated_icon!(FileCog);
    def_animated_icon!(FileDiff);
    def_animated_icon!(FileDigit);
    def_animated_icon!(FileDown);
    def_animated_icon!(FileExclamationPoint);
    def_animated_icon!(FileHeadphone);
    def_animated_icon!(FileHeart);
    def_animated_icon!(FileImage);
    def_animated_icon!(FileInput);
    def_animated_icon!(FileKey);
    def_animated_icon!(FileLock);
    def_animated_icon!(FileMinusCorner);
    def_animated_icon!(FileMinus);
    def_animated_icon!(FileMusic);
    def_animated_icon!(FileOutput);
    def_animated_icon!(FilePenLine);
    def_animated_icon!(FilePen);
    def_animated_icon!(FilePlay);
    def_animated_icon!(FilePlusCorner);
    def_animated_icon!(FilePlus);
    def_animated_icon!(FileQuestionMark);
    def_animated_icon!(FileScan);
    def_animated_icon!(FileSearchCorner);
    def_animated_icon!(FileSearch);
    def_animated_icon!(FileSignal);
    def_animated_icon!(FileSliders);
    def_animated_icon!(FileSpreadsheet);
    def_animated_icon!(FileStack);
    def_animated_icon!(FileSymlink);
    def_animated_icon!(FileTerminal);
    def_animated_icon!(FileText);
    def_animated_icon!(FileTypeCorner);
    def_animated_icon!(FileType);
    def_animated_icon!(FileUp);
    def_animated_icon!(FileUser);
    def_animated_icon!(FileVideoCamera);
    def_animated_icon!(FileVolume);
    def_animated_icon!(FileXCorner);
    def_animated_icon!(FileX);
    def_animated_icon!(File);
    def_animated_icon!(Files);
    def_animated_icon!(Film);
    def_animated_icon!(FingerprintPattern);
    def_animated_icon!(FireExtinguisher);
    def_animated_icon!(FishOff);
    def_animated_icon!(FishSymbol);
    def_animated_icon!(Fish);
    def_animated_icon!(FishingHook);
    def_animated_icon!(FishingRod);
    def_animated_icon!(FlagOff);
    def_animated_icon!(FlagTriangleLeft);
    def_animated_icon!(FlagTriangleRight);
    def_animated_icon!(Flag);
    def_animated_icon!(FlameKindling);
    def_animated_icon!(Flame);
    def_animated_icon!(FlashlightOff);
    def_animated_icon!(Flashlight);
    def_animated_icon!(FlaskConicalOff);
    def_animated_icon!(FlaskConical);
    def_animated_icon!(FlaskRound);
    def_animated_icon!(FlipHorizontal2);
    def_animated_icon!(FlipVertical2);
    def_animated_icon!(Flower2);
    def_animated_icon!(Flower);
    def_animated_icon!(Focus);
    def_animated_icon!(FoldHorizontal);
    def_animated_icon!(FoldVertical);
    def_animated_icon!(FolderArchive);
    def_animated_icon!(FolderBookmark);
    def_animated_icon!(FolderCheck);
    def_animated_icon!(FolderClock);
    def_animated_icon!(FolderClosed);
    def_animated_icon!(FolderCode);
    def_animated_icon!(FolderCog);
    def_animated_icon!(FolderDot);
    def_animated_icon!(FolderDown);
    def_animated_icon!(FolderGit2);
    def_animated_icon!(FolderGit);
    def_animated_icon!(FolderHeart);
    def_animated_icon!(FolderInput);
    def_animated_icon!(FolderKanban);
    def_animated_icon!(FolderKey);
    def_animated_icon!(FolderLock);
    def_animated_icon!(FolderMinus);
    def_animated_icon!(FolderOpenDot);
    def_animated_icon!(FolderOpen);
    def_animated_icon!(FolderOutput);
    def_animated_icon!(FolderPen);
    def_animated_icon!(FolderPlus);
    def_animated_icon!(FolderRoot);
    def_animated_icon!(FolderSearch2);
    def_animated_icon!(FolderSearch);
    def_animated_icon!(FolderSymlink);
    def_animated_icon!(FolderSync);
    def_animated_icon!(FolderTree);
    def_animated_icon!(FolderUp);
    def_animated_icon!(FolderX);
    def_animated_icon!(Folder);
    def_animated_icon!(Folders);
    def_animated_icon!(Footprints);
    def_animated_icon!(Forklift);
    def_animated_icon!(Form);
    def_animated_icon!(Forward);
    def_animated_icon!(Frame);
    def_animated_icon!(Frown);
    def_animated_icon!(Fuel);
    def_animated_icon!(Fullscreen);
    def_animated_icon!(FunnelPlus);
    def_animated_icon!(FunnelX);
    def_animated_icon!(Funnel);
    def_animated_icon!(GalleryHorizontalEnd);
    def_animated_icon!(GalleryHorizontal);
    def_animated_icon!(GalleryThumbnails);
    def_animated_icon!(GalleryVerticalEnd);
    def_animated_icon!(GalleryVertical);
    def_animated_icon!(Gamepad2);
    def_animated_icon!(GamepadDirectional);
    def_animated_icon!(Gamepad);
    def_animated_icon!(Gauge);
    def_animated_icon!(Gavel);
    def_animated_icon!(Gem);
    def_animated_icon!(GeorgianLari);
    def_animated_icon!(Ghost);
    def_animated_icon!(Gift);
    def_animated_icon!(GitBranchMinus);
    def_animated_icon!(GitBranchPlus);
    def_animated_icon!(GitBranch);
    def_animated_icon!(GitCommitHorizontal);
    def_animated_icon!(GitCommitVertical);
    def_animated_icon!(GitCompareArrows);
    def_animated_icon!(GitCompare);
    def_animated_icon!(GitFork);
    def_animated_icon!(GitGraph);
    def_animated_icon!(GitMergeConflict);
    def_animated_icon!(GitMerge);
    def_animated_icon!(GitPullRequestArrow);
    def_animated_icon!(GitPullRequestClosed);
    def_animated_icon!(GitPullRequestCreateArrow);
    def_animated_icon!(GitPullRequestCreate);
    def_animated_icon!(GitPullRequestDraft);
    def_animated_icon!(GitPullRequest);
    def_animated_icon!(GlassWater);
    def_animated_icon!(Glasses);
    def_animated_icon!(GlobeCheck);
    def_animated_icon!(GlobeLock);
    def_animated_icon!(GlobeOff);
    def_animated_icon!(GlobeX);
    def_animated_icon!(Globe);
    def_animated_icon!(Goal);
    def_animated_icon!(Gpu);
    def_animated_icon!(GraduationCap);
    def_animated_icon!(Grape);
    def_animated_icon!(Grid2X2Check);
    def_animated_icon!(Grid2X2Plus);
    def_animated_icon!(Grid2X2X);
    def_animated_icon!(Grid2X2);
    def_animated_icon!(Grid3X2);
    def_animated_icon!(Grid3X3);
    def_animated_icon!(GripHorizontal);
    def_animated_icon!(GripVertical);
    def_animated_icon!(Grip);
    def_animated_icon!(Group);
    def_animated_icon!(Guitar);
    def_animated_icon!(Ham);
    def_animated_icon!(Hamburger);
    def_animated_icon!(Hammer);
    def_animated_icon!(HandCoins);
    def_animated_icon!(HandFist);
    def_animated_icon!(HandGrab);
    def_animated_icon!(HandHeart);
    def_animated_icon!(HandHelping);
    def_animated_icon!(HandMetal);
    def_animated_icon!(HandPlatter);
    def_animated_icon!(Hand);
    def_animated_icon!(Handbag);
    def_animated_icon!(Handshake);
    def_animated_icon!(HardDriveDownload);
    def_animated_icon!(HardDriveUpload);
    def_animated_icon!(HardDrive);
    def_animated_icon!(HardHat);
    def_animated_icon!(Hash);
    def_animated_icon!(HatGlasses);
    def_animated_icon!(Haze);
    def_animated_icon!(Hd);
    def_animated_icon!(HdmiPort);
    def_animated_icon!(Heading1);
    def_animated_icon!(Heading2);
    def_animated_icon!(Heading3);
    def_animated_icon!(Heading4);
    def_animated_icon!(Heading5);
    def_animated_icon!(Heading6);
    def_animated_icon!(Heading);
    def_animated_icon!(HeadphoneOff);
    def_animated_icon!(Headphones);
    def_animated_icon!(Headset);
    def_animated_icon!(HeartCrack);
    def_animated_icon!(HeartHandshake);
    def_animated_icon!(HeartMinus);
    def_animated_icon!(HeartOff);
    def_animated_icon!(HeartPlus);
    def_animated_icon!(HeartPulse);
    def_animated_icon!(HeartX);
    def_animated_icon!(Heart);
    def_animated_icon!(Heater);
    def_animated_icon!(Helicopter);
    def_animated_icon!(Hexagon);
    def_animated_icon!(Highlighter);
    def_animated_icon!(History);
    def_animated_icon!(HopOff);
    def_animated_icon!(Hop);
    def_animated_icon!(Hospital);
    def_animated_icon!(Hotel);
    def_animated_icon!(Hourglass);
    def_animated_icon!(HouseHeart);
    def_animated_icon!(HousePlug);
    def_animated_icon!(HousePlus);
    def_animated_icon!(HouseWifi);
    def_animated_icon!(House);
    def_animated_icon!(IceCreamBowl);
    def_animated_icon!(IceCreamCone);
    def_animated_icon!(IdCardLanyard);
    def_animated_icon!(IdCard);
    def_animated_icon!(ImageDown);
    def_animated_icon!(ImageMinus);
    def_animated_icon!(ImageOff);
    def_animated_icon!(ImagePlay);
    def_animated_icon!(ImagePlus);
    def_animated_icon!(ImageUp);
    def_animated_icon!(ImageUpscale);
    def_animated_icon!(Image);
    def_animated_icon!(Images);
    def_animated_icon!(Import);
    def_animated_icon!(Inbox);
    def_animated_icon!(IndianRupee);
    def_animated_icon!(Infinity);
    def_animated_icon!(Info);
    def_animated_icon!(InspectionPanel);
    def_animated_icon!(Italic);
    def_animated_icon!(IterationCcw);
    def_animated_icon!(IterationCw);
    def_animated_icon!(JapaneseYen);
    def_animated_icon!(Joystick);
    def_animated_icon!(Kanban);
    def_animated_icon!(Kayak);
    def_animated_icon!(KeyRound);
    def_animated_icon!(KeySquare);
    def_animated_icon!(Key);
    def_animated_icon!(KeyboardMusic);
    def_animated_icon!(KeyboardOff);
    def_animated_icon!(Keyboard);
    def_animated_icon!(LampCeiling);
    def_animated_icon!(LampDesk);
    def_animated_icon!(LampFloor);
    def_animated_icon!(LampWallDown);
    def_animated_icon!(LampWallUp);
    def_animated_icon!(Lamp);
    def_animated_icon!(LandPlot);
    def_animated_icon!(Landmark);
    def_animated_icon!(Languages);
    def_animated_icon!(LaptopMinimalCheck);
    def_animated_icon!(LaptopMinimal);
    def_animated_icon!(Laptop);
    def_animated_icon!(LassoSelect);
    def_animated_icon!(Lasso);
    def_animated_icon!(Laugh);
    def_animated_icon!(Layers2);
    def_animated_icon!(LayersMinus);
    def_animated_icon!(LayersPlus);
    def_animated_icon!(Layers);
    def_animated_icon!(LayoutDashboard);
    def_animated_icon!(LayoutGrid);
    def_animated_icon!(LayoutList);
    def_animated_icon!(LayoutPanelLeft);
    def_animated_icon!(LayoutPanelTop);
    def_animated_icon!(LayoutTemplate);
    def_animated_icon!(Leaf);
    def_animated_icon!(LeafyGreen);
    def_animated_icon!(Lectern);
    def_animated_icon!(LensConcave);
    def_animated_icon!(LensConvex);
    def_animated_icon!(LibraryBig);
    def_animated_icon!(Library);
    def_animated_icon!(LifeBuoy);
    def_animated_icon!(Ligature);
    def_animated_icon!(LightbulbOff);
    def_animated_icon!(Lightbulb);
    def_animated_icon!(LineDotRightHorizontal);
    def_animated_icon!(LineSquiggle);
    def_animated_icon!(LineStyle);
    def_animated_icon!(Link2Off);
    def_animated_icon!(Link2);
    def_animated_icon!(Link);
    def_animated_icon!(ListCheck);
    def_animated_icon!(ListChecks);
    def_animated_icon!(ListChevronsDownUp);
    def_animated_icon!(ListChevronsUpDown);
    def_animated_icon!(ListCollapse);
    def_animated_icon!(ListEnd);
    def_animated_icon!(ListFilterPlus);
    def_animated_icon!(ListFilter);
    def_animated_icon!(ListIndentDecrease);
    def_animated_icon!(ListIndentIncrease);
    def_animated_icon!(ListMinus);
    def_animated_icon!(ListMusic);
    def_animated_icon!(ListOrdered);
    def_animated_icon!(ListPlus);
    def_animated_icon!(ListRestart);
    def_animated_icon!(ListSortAscending);
    def_animated_icon!(ListSortDescending);
    def_animated_icon!(ListStart);
    def_animated_icon!(ListTodo);
    def_animated_icon!(ListTree);
    def_animated_icon!(ListVideo);
    def_animated_icon!(ListX);
    def_animated_icon!(List);
    def_animated_icon!(LoaderCircle);
    def_animated_icon!(LoaderPinwheel);
    def_animated_icon!(Loader);
    def_animated_icon!(LocateFixed);
    def_animated_icon!(LocateOff);
    def_animated_icon!(Locate);
    def_animated_icon!(LockKeyholeOpen);
    def_animated_icon!(LockKeyhole);
    def_animated_icon!(LockOpen);
    def_animated_icon!(Lock);
    def_animated_icon!(LogIn);
    def_animated_icon!(LogOut);
    def_animated_icon!(Logs);
    def_animated_icon!(Lollipop);
    def_animated_icon!(Luggage);
    def_animated_icon!(Magnet);
    def_animated_icon!(MailCheck);
    def_animated_icon!(MailMinus);
    def_animated_icon!(MailOpen);
    def_animated_icon!(MailPlus);
    def_animated_icon!(MailQuestionMark);
    def_animated_icon!(MailSearch);
    def_animated_icon!(MailWarning);
    def_animated_icon!(MailX);
    def_animated_icon!(Mail);
    def_animated_icon!(Mailbox);
    def_animated_icon!(Mails);
    def_animated_icon!(MapMinus);
    def_animated_icon!(MapPinCheckInside);
    def_animated_icon!(MapPinCheck);
    def_animated_icon!(MapPinHouse);
    def_animated_icon!(MapPinMinusInside);
    def_animated_icon!(MapPinMinus);
    def_animated_icon!(MapPinOff);
    def_animated_icon!(MapPinPen);
    def_animated_icon!(MapPinPlusInside);
    def_animated_icon!(MapPinPlus);
    def_animated_icon!(MapPinSearch);
    def_animated_icon!(MapPinXInside);
    def_animated_icon!(MapPinX);
    def_animated_icon!(MapPin);
    def_animated_icon!(MapPinned);
    def_animated_icon!(MapPlus);
    def_animated_icon!(Map);
    def_animated_icon!(MarsStroke);
    def_animated_icon!(Mars);
    def_animated_icon!(Martini);
    def_animated_icon!(Maximize2);
    def_animated_icon!(Maximize);
    def_animated_icon!(Medal);
    def_animated_icon!(MegaphoneOff);
    def_animated_icon!(Megaphone);
    def_animated_icon!(Meh);
    def_animated_icon!(MemoryStick);
    def_animated_icon!(Menu);
    def_animated_icon!(Merge);
    def_animated_icon!(MessageCircleCheck);
    def_animated_icon!(MessageCircleCode);
    def_animated_icon!(MessageCircleDashed);
    def_animated_icon!(MessageCircleHeart);
    def_animated_icon!(MessageCircleMore);
    def_animated_icon!(MessageCircleOff);
    def_animated_icon!(MessageCirclePlus);
    def_animated_icon!(MessageCircleQuestionMark);
    def_animated_icon!(MessageCircleReply);
    def_animated_icon!(MessageCircleWarning);
    def_animated_icon!(MessageCircleX);
    def_animated_icon!(MessageCircle);
    def_animated_icon!(MessageSquareCheck);
    def_animated_icon!(MessageSquareCode);
    def_animated_icon!(MessageSquareDashed);
    def_animated_icon!(MessageSquareDiff);
    def_animated_icon!(MessageSquareDot);
    def_animated_icon!(MessageSquareHeart);
    def_animated_icon!(MessageSquareLock);
    def_animated_icon!(MessageSquareMore);
    def_animated_icon!(MessageSquareOff);
    def_animated_icon!(MessageSquarePlus);
    def_animated_icon!(MessageSquareQuote);
    def_animated_icon!(MessageSquareReply);
    def_animated_icon!(MessageSquareShare);
    def_animated_icon!(MessageSquareText);
    def_animated_icon!(MessageSquareWarning);
    def_animated_icon!(MessageSquareX);
    def_animated_icon!(MessageSquare);
    def_animated_icon!(MessagesSquare);
    def_animated_icon!(Metronome);
    def_animated_icon!(MicOff);
    def_animated_icon!(MicVocal);
    def_animated_icon!(Mic);
    def_animated_icon!(Microchip);
    def_animated_icon!(Microscope);
    def_animated_icon!(Microwave);
    def_animated_icon!(Milestone);
    def_animated_icon!(MilkOff);
    def_animated_icon!(Milk);
    def_animated_icon!(Minimize2);
    def_animated_icon!(Minimize);
    def_animated_icon!(Minus);
    def_animated_icon!(MirrorRectangular);
    def_animated_icon!(MirrorRound);
    def_animated_icon!(MonitorCheck);
    def_animated_icon!(MonitorCloud);
    def_animated_icon!(MonitorCog);
    def_animated_icon!(MonitorDot);
    def_animated_icon!(MonitorDown);
    def_animated_icon!(MonitorOff);
    def_animated_icon!(MonitorPause);
    def_animated_icon!(MonitorPlay);
    def_animated_icon!(MonitorSmartphone);
    def_animated_icon!(MonitorSpeaker);
    def_animated_icon!(MonitorStop);
    def_animated_icon!(MonitorUp);
    def_animated_icon!(MonitorX);
    def_animated_icon!(Monitor);
    def_animated_icon!(MoonStar);
    def_animated_icon!(Moon);
    def_animated_icon!(Motorbike);
    def_animated_icon!(MountainSnow);
    def_animated_icon!(Mountain);
    def_animated_icon!(MouseLeft);
    def_animated_icon!(MouseOff);
    def_animated_icon!(MousePointer2Off);
    def_animated_icon!(MousePointer2);
    def_animated_icon!(MousePointerBan);
    def_animated_icon!(MousePointerClick);
    def_animated_icon!(MousePointer);
    def_animated_icon!(MouseRight);
    def_animated_icon!(Mouse);
    def_animated_icon!(Move3D);
    def_animated_icon!(MoveDiagonal2);
    def_animated_icon!(MoveDiagonal);
    def_animated_icon!(MoveDownLeft);
    def_animated_icon!(MoveDownRight);
    def_animated_icon!(MoveDown);
    def_animated_icon!(MoveHorizontal);
    def_animated_icon!(MoveLeft);
    def_animated_icon!(MoveRight);
    def_animated_icon!(MoveUpLeft);
    def_animated_icon!(MoveUpRight);
    def_animated_icon!(MoveUp);
    def_animated_icon!(MoveVertical);
    def_animated_icon!(Move);
    def_animated_icon!(Music2);
    def_animated_icon!(Music3);
    def_animated_icon!(Music4);
    def_animated_icon!(Music);
    def_animated_icon!(Navigation2Off);
    def_animated_icon!(Navigation2);
    def_animated_icon!(NavigationOff);
    def_animated_icon!(Navigation);
    def_animated_icon!(Network);
    def_animated_icon!(Newspaper);
    def_animated_icon!(Nfc);
    def_animated_icon!(NonBinary);
    def_animated_icon!(NotebookPen);
    def_animated_icon!(NotebookTabs);
    def_animated_icon!(NotebookText);
    def_animated_icon!(Notebook);
    def_animated_icon!(NotepadTextDashed);
    def_animated_icon!(NotepadText);
    def_animated_icon!(NutOff);
    def_animated_icon!(Nut);
    def_animated_icon!(OctagonAlert);
    def_animated_icon!(OctagonMinus);
    def_animated_icon!(OctagonPause);
    def_animated_icon!(OctagonX);
    def_animated_icon!(Octagon);
    def_animated_icon!(Omega);
    def_animated_icon!(Option);
    def_animated_icon!(Orbit);
    def_animated_icon!(Origami);
    def_animated_icon!(Package2);
    def_animated_icon!(PackageCheck);
    def_animated_icon!(PackageMinus);
    def_animated_icon!(PackageOpen);
    def_animated_icon!(PackagePlus);
    def_animated_icon!(PackageSearch);
    def_animated_icon!(PackageX);
    def_animated_icon!(Package);
    def_animated_icon!(PaintBucket);
    def_animated_icon!(PaintRoller);
    def_animated_icon!(PaintbrushVertical);
    def_animated_icon!(Paintbrush);
    def_animated_icon!(Palette);
    def_animated_icon!(Panda);
    def_animated_icon!(PanelBottomClose);
    def_animated_icon!(PanelBottomDashed);
    def_animated_icon!(PanelBottomOpen);
    def_animated_icon!(PanelBottom);
    def_animated_icon!(PanelLeftClose);
    def_animated_icon!(PanelLeftDashed);
    def_animated_icon!(PanelLeftOpen);
    def_animated_icon!(PanelLeftRightDashed);
    def_animated_icon!(PanelLeft);
    def_animated_icon!(PanelRightClose);
    def_animated_icon!(PanelRightDashed);
    def_animated_icon!(PanelRightOpen);
    def_animated_icon!(PanelRight);
    def_animated_icon!(PanelTopBottomDashed);
    def_animated_icon!(PanelTopClose);
    def_animated_icon!(PanelTopDashed);
    def_animated_icon!(PanelTopOpen);
    def_animated_icon!(PanelTop);
    def_animated_icon!(PanelsLeftBottom);
    def_animated_icon!(PanelsRightBottom);
    def_animated_icon!(PanelsTopLeft);
    def_animated_icon!(Paperclip);
    def_animated_icon!(Parasol);
    def_animated_icon!(Parentheses);
    def_animated_icon!(ParkingMeter);
    def_animated_icon!(PartyPopper);
    def_animated_icon!(Pause);
    def_animated_icon!(PawPrint);
    def_animated_icon!(PcCase);
    def_animated_icon!(PenLine);
    def_animated_icon!(PenOff);
    def_animated_icon!(PenTool);
    def_animated_icon!(Pen);
    def_animated_icon!(PencilLine);
    def_animated_icon!(PencilOff);
    def_animated_icon!(PencilRuler);
    def_animated_icon!(PencilSparkles);
    def_animated_icon!(Pencil);
    def_animated_icon!(Pentagon);
    def_animated_icon!(Percent);
    def_animated_icon!(PersonStanding);
    def_animated_icon!(PhilippinePeso);
    def_animated_icon!(PhoneCall);
    def_animated_icon!(PhoneForwarded);
    def_animated_icon!(PhoneIncoming);
    def_animated_icon!(PhoneMissed);
    def_animated_icon!(PhoneOff);
    def_animated_icon!(PhoneOutgoing);
    def_animated_icon!(Phone);
    def_animated_icon!(Pi);
    def_animated_icon!(Piano);
    def_animated_icon!(Pickaxe);
    def_animated_icon!(PictureInPicture2);
    def_animated_icon!(PictureInPicture);
    def_animated_icon!(PiggyBank);
    def_animated_icon!(PilcrowLeft);
    def_animated_icon!(PilcrowRight);
    def_animated_icon!(Pilcrow);
    def_animated_icon!(PillBottle);
    def_animated_icon!(Pill);
    def_animated_icon!(PinOff);
    def_animated_icon!(Pin);
    def_animated_icon!(Pipette);
    def_animated_icon!(Pizza);
    def_animated_icon!(PlaneLanding);
    def_animated_icon!(PlaneTakeoff);
    def_animated_icon!(Plane);
    def_animated_icon!(PlayOff);
    def_animated_icon!(Play);
    def_animated_icon!(Plug2);
    def_animated_icon!(PlugZap);
    def_animated_icon!(Plug);
    def_animated_icon!(Plus);
    def_animated_icon!(PocketKnife);
    def_animated_icon!(Podcast);
    def_animated_icon!(Podium);
    def_animated_icon!(PointerOff);
    def_animated_icon!(Pointer);
    def_animated_icon!(Popcorn);
    def_animated_icon!(Popsicle);
    def_animated_icon!(PoundSterling);
    def_animated_icon!(PowerOff);
    def_animated_icon!(Power);
    def_animated_icon!(Presentation);
    def_animated_icon!(PrinterCheck);
    def_animated_icon!(PrinterX);
    def_animated_icon!(Printer);
    def_animated_icon!(Projector);
    def_animated_icon!(Proportions);
    def_animated_icon!(Puzzle);
    def_animated_icon!(Pyramid);
    def_animated_icon!(QrCode);
    def_animated_icon!(Quote);
    def_animated_icon!(Rabbit);
    def_animated_icon!(Radar);
    def_animated_icon!(Radiation);
    def_animated_icon!(Radical);
    def_animated_icon!(RadioOff);
    def_animated_icon!(RadioReceiver);
    def_animated_icon!(RadioTower);
    def_animated_icon!(Radio);
    def_animated_icon!(Radius);
    def_animated_icon!(Rainbow);
    def_animated_icon!(Rat);
    def_animated_icon!(Ratio);
    def_animated_icon!(ReceiptCent);
    def_animated_icon!(ReceiptEuro);
    def_animated_icon!(ReceiptIndianRupee);
    def_animated_icon!(ReceiptJapaneseYen);
    def_animated_icon!(ReceiptPoundSterling);
    def_animated_icon!(ReceiptRussianRuble);
    def_animated_icon!(ReceiptSwissFranc);
    def_animated_icon!(ReceiptText);
    def_animated_icon!(ReceiptTurkishLira);
    def_animated_icon!(Receipt);
    def_animated_icon!(RectangleCircle);
    def_animated_icon!(RectangleEllipsis);
    def_animated_icon!(RectangleGoggles);
    def_animated_icon!(RectangleHorizontal);
    def_animated_icon!(RectangleVertical);
    def_animated_icon!(Recycle);
    def_animated_icon!(Redo2);
    def_animated_icon!(RedoDot);
    def_animated_icon!(Redo);
    def_animated_icon!(RefreshCcwDot);
    def_animated_icon!(RefreshCcw);
    def_animated_icon!(RefreshCwOff);
    def_animated_icon!(RefreshCw);
    def_animated_icon!(Refrigerator);
    def_animated_icon!(Regex);
    def_animated_icon!(RemoveFormatting);
    def_animated_icon!(Repeat1);
    def_animated_icon!(Repeat2);
    def_animated_icon!(RepeatOff);
    def_animated_icon!(Repeat);
    def_animated_icon!(ReplaceAll);
    def_animated_icon!(Replace);
    def_animated_icon!(ReplyAll);
    def_animated_icon!(Reply);
    def_animated_icon!(Rewind);
    def_animated_icon!(Ribbon);
    def_animated_icon!(Road);
    def_animated_icon!(Rocket);
    def_animated_icon!(RockingChair);
    def_animated_icon!(RollerCoaster);
    def_animated_icon!(Rose);
    def_animated_icon!(Rotate3D);
    def_animated_icon!(RotateCcwKey);
    def_animated_icon!(RotateCcwSquare);
    def_animated_icon!(RotateCcw);
    def_animated_icon!(RotateCwSquare);
    def_animated_icon!(RotateCw);
    def_animated_icon!(RouteOff);
    def_animated_icon!(Route);
    def_animated_icon!(Router);
    def_animated_icon!(Rows2);
    def_animated_icon!(Rows3);
    def_animated_icon!(Rows4);
    def_animated_icon!(Rss);
    def_animated_icon!(RulerDimensionLine);
    def_animated_icon!(Ruler);
    def_animated_icon!(RussianRuble);
    def_animated_icon!(Sailboat);
    def_animated_icon!(Salad);
    def_animated_icon!(Sandwich);
    def_animated_icon!(SatelliteDish);
    def_animated_icon!(Satellite);
    def_animated_icon!(SaudiRiyal);
    def_animated_icon!(SaveAll);
    def_animated_icon!(SaveCheck);
    def_animated_icon!(SaveOff);
    def_animated_icon!(SavePen);
    def_animated_icon!(SavePlus);
    def_animated_icon!(Save);
    def_animated_icon!(Scale3D);
    def_animated_icon!(Scale);
    def_animated_icon!(Scaling);
    def_animated_icon!(ScanBarcode);
    def_animated_icon!(ScanEye);
    def_animated_icon!(ScanFace);
    def_animated_icon!(ScanHeart);
    def_animated_icon!(ScanLine);
    def_animated_icon!(ScanQrCode);
    def_animated_icon!(ScanSearch);
    def_animated_icon!(ScanText);
    def_animated_icon!(Scan);
    def_animated_icon!(School);
    def_animated_icon!(ScissorsLineDashed);
    def_animated_icon!(Scissors);
    def_animated_icon!(Scooter);
    def_animated_icon!(ScreenShareOff);
    def_animated_icon!(ScreenShare);
    def_animated_icon!(ScrollText);
    def_animated_icon!(Scroll);
    def_animated_icon!(SearchAlert);
    def_animated_icon!(SearchCheck);
    def_animated_icon!(SearchCode);
    def_animated_icon!(SearchSlash);
    def_animated_icon!(SearchX);
    def_animated_icon!(Search);
    def_animated_icon!(Section);
    def_animated_icon!(SendHorizontal);
    def_animated_icon!(SendToBack);
    def_animated_icon!(Send);
    def_animated_icon!(SeparatorHorizontal);
    def_animated_icon!(SeparatorVertical);
    def_animated_icon!(ServerCog);
    def_animated_icon!(ServerCrash);
    def_animated_icon!(ServerOff);
    def_animated_icon!(Server);
    def_animated_icon!(Settings2);
    def_animated_icon!(Settings);
    def_animated_icon!(Shapes);
    def_animated_icon!(Share2);
    def_animated_icon!(Share);
    def_animated_icon!(Sheet);
    def_animated_icon!(Shell);
    def_animated_icon!(ShelvingUnit);
    def_animated_icon!(ShieldAlert);
    def_animated_icon!(ShieldBan);
    def_animated_icon!(ShieldCheck);
    def_animated_icon!(ShieldCogCorner);
    def_animated_icon!(ShieldCog);
    def_animated_icon!(ShieldEllipsis);
    def_animated_icon!(ShieldHalf);
    def_animated_icon!(ShieldMinus);
    def_animated_icon!(ShieldOff);
    def_animated_icon!(ShieldPlus);
    def_animated_icon!(ShieldQuestionMark);
    def_animated_icon!(ShieldUser);
    def_animated_icon!(ShieldX);
    def_animated_icon!(Shield);
    def_animated_icon!(ShipWheel);
    def_animated_icon!(Ship);
    def_animated_icon!(Shirt);
    def_animated_icon!(ShoppingBag);
    def_animated_icon!(ShoppingBasket);
    def_animated_icon!(ShoppingCart);
    def_animated_icon!(Shovel);
    def_animated_icon!(ShowerHead);
    def_animated_icon!(Shredder);
    def_animated_icon!(Shrimp);
    def_animated_icon!(Shrink);
    def_animated_icon!(Shrub);
    def_animated_icon!(Shuffle);
    def_animated_icon!(Sigma);
    def_animated_icon!(SignalHigh);
    def_animated_icon!(SignalLow);
    def_animated_icon!(SignalMedium);
    def_animated_icon!(SignalZero);
    def_animated_icon!(Signal);
    def_animated_icon!(Signature);
    def_animated_icon!(SignpostBig);
    def_animated_icon!(Signpost);
    def_animated_icon!(Siren);
    def_animated_icon!(SkipBack);
    def_animated_icon!(SkipForward);
    def_animated_icon!(Skull);
    def_animated_icon!(Slash);
    def_animated_icon!(Slice);
    def_animated_icon!(SlidersHorizontal);
    def_animated_icon!(SlidersVertical);
    def_animated_icon!(SmartphoneCharging);
    def_animated_icon!(SmartphoneNfc);
    def_animated_icon!(Smartphone);
    def_animated_icon!(SmilePlus);
    def_animated_icon!(Smile);
    def_animated_icon!(Snail);
    def_animated_icon!(Snowflake);
    def_animated_icon!(SoapDispenserDroplet);
    def_animated_icon!(Sofa);
    def_animated_icon!(SolarPanel);
    def_animated_icon!(Soup);
    def_animated_icon!(Space);
    def_animated_icon!(Spade);
    def_animated_icon!(Sparkle);
    def_animated_icon!(Sparkles);
    def_animated_icon!(Speaker);
    def_animated_icon!(Speech);
    def_animated_icon!(SpellCheck2);
    def_animated_icon!(SpellCheck);
    def_animated_icon!(SplinePointer);
    def_animated_icon!(Spline);
    def_animated_icon!(Split);
    def_animated_icon!(Spool);
    def_animated_icon!(SportShoe);
    def_animated_icon!(Spotlight);
    def_animated_icon!(SprayCan);
    def_animated_icon!(Sprout);
    def_animated_icon!(SquareActivity);
    def_animated_icon!(SquareArrowDownLeft);
    def_animated_icon!(SquareArrowDownRight);
    def_animated_icon!(SquareArrowDown);
    def_animated_icon!(SquareArrowLeft);
    def_animated_icon!(SquareArrowOutDownLeft);
    def_animated_icon!(SquareArrowOutDownRight);
    def_animated_icon!(SquareArrowOutUpLeft);
    def_animated_icon!(SquareArrowOutUpRight);
    def_animated_icon!(SquareArrowRightEnter);
    def_animated_icon!(SquareArrowRightExit);
    def_animated_icon!(SquareArrowRight);
    def_animated_icon!(SquareArrowUpLeft);
    def_animated_icon!(SquareArrowUpRight);
    def_animated_icon!(SquareArrowUp);
    def_animated_icon!(SquareAsterisk);
    def_animated_icon!(SquareBottomDashedScissors);
    def_animated_icon!(SquareCenterlineDashedHorizontal);
    def_animated_icon!(SquareCenterlineDashedVertical);
    def_animated_icon!(SquareChartGantt);
    def_animated_icon!(SquareCheckBig);
    def_animated_icon!(SquareCheck);
    def_animated_icon!(SquareChevronDown);
    def_animated_icon!(SquareChevronLeft);
    def_animated_icon!(SquareChevronRight);
    def_animated_icon!(SquareChevronUp);
    def_animated_icon!(SquareCode);
    def_animated_icon!(SquareDashedBottomCode);
    def_animated_icon!(SquareDashedBottom);
    def_animated_icon!(SquareDashedKanban);
    def_animated_icon!(SquareDashedMousePointer);
    def_animated_icon!(SquareDashedText);
    def_animated_icon!(SquareDashedTopSolid);
    def_animated_icon!(SquareDashed);
    def_animated_icon!(SquareDivide);
    def_animated_icon!(SquareDot);
    def_animated_icon!(SquareEqual);
    def_animated_icon!(SquareFunction);
    def_animated_icon!(SquareKanban);
    def_animated_icon!(SquareLibrary);
    def_animated_icon!(SquareM);
    def_animated_icon!(SquareMenu);
    def_animated_icon!(SquareMinus);
    def_animated_icon!(SquareMousePointer);
    def_animated_icon!(SquareParkingOff);
    def_animated_icon!(SquareParking);
    def_animated_icon!(SquarePause);
    def_animated_icon!(SquarePen);
    def_animated_icon!(SquarePercent);
    def_animated_icon!(SquarePi);
    def_animated_icon!(SquarePilcrow);
    def_animated_icon!(SquarePlay);
    def_animated_icon!(SquarePlus);
    def_animated_icon!(SquarePower);
    def_animated_icon!(SquareRadical);
    def_animated_icon!(SquareRoundCorner);
    def_animated_icon!(SquareScissors);
    def_animated_icon!(SquareSigma);
    def_animated_icon!(SquareSlash);
    def_animated_icon!(SquareSplitHorizontal);
    def_animated_icon!(SquareSplitVertical);
    def_animated_icon!(SquareSquare);
    def_animated_icon!(SquareStack);
    def_animated_icon!(SquareStar);
    def_animated_icon!(SquareStop);
    def_animated_icon!(SquareTerminal);
    def_animated_icon!(SquareUserRound);
    def_animated_icon!(SquareUser);
    def_animated_icon!(SquareX);
    def_animated_icon!(Square);
    def_animated_icon!(SquaresExclude);
    def_animated_icon!(SquaresIntersect);
    def_animated_icon!(SquaresSubtract);
    def_animated_icon!(SquaresUnite);
    def_animated_icon!(SquircleDashed);
    def_animated_icon!(Squircle);
    def_animated_icon!(Squirrel);
    def_animated_icon!(Stamp);
    def_animated_icon!(StarCheck);
    def_animated_icon!(StarHalf);
    def_animated_icon!(StarMinus);
    def_animated_icon!(StarOff);
    def_animated_icon!(StarPlus);
    def_animated_icon!(StarX);
    def_animated_icon!(Star);
    def_animated_icon!(StepBack);
    def_animated_icon!(StepForward);
    def_animated_icon!(Stethoscope);
    def_animated_icon!(Sticker);
    def_animated_icon!(StickyNoteCheck);
    def_animated_icon!(StickyNoteMinus);
    def_animated_icon!(StickyNoteOff);
    def_animated_icon!(StickyNotePlus);
    def_animated_icon!(StickyNoteX);
    def_animated_icon!(StickyNote);
    def_animated_icon!(StickyNotes);
    def_animated_icon!(Stone);
    def_animated_icon!(Store);
    def_animated_icon!(StretchHorizontal);
    def_animated_icon!(StretchVertical);
    def_animated_icon!(Strikethrough);
    def_animated_icon!(Subscript);
    def_animated_icon!(Summary);
    def_animated_icon!(SunDim);
    def_animated_icon!(SunMedium);
    def_animated_icon!(SunMoon);
    def_animated_icon!(SunSnow);
    def_animated_icon!(Sun);
    def_animated_icon!(Sunrise);
    def_animated_icon!(Sunset);
    def_animated_icon!(Superscript);
    def_animated_icon!(SwatchBook);
    def_animated_icon!(SwissFranc);
    def_animated_icon!(SwitchCamera);
    def_animated_icon!(Sword);
    def_animated_icon!(Swords);
    def_animated_icon!(Syringe);
    def_animated_icon!(Table2);
    def_animated_icon!(TableCellsMerge);
    def_animated_icon!(TableCellsSplit);
    def_animated_icon!(TableColumnsSplit);
    def_animated_icon!(TableOfContents);
    def_animated_icon!(TableProperties);
    def_animated_icon!(TableRowsSplit);
    def_animated_icon!(Table);
    def_animated_icon!(TabletSmartphone);
    def_animated_icon!(Tablet);
    def_animated_icon!(Tablets);
    def_animated_icon!(TagPlus);
    def_animated_icon!(TagX);
    def_animated_icon!(Tag);
    def_animated_icon!(Tags);
    def_animated_icon!(Tally1);
    def_animated_icon!(Tally2);
    def_animated_icon!(Tally3);
    def_animated_icon!(Tally4);
    def_animated_icon!(Tally5);
    def_animated_icon!(Tangent);
    def_animated_icon!(Target);
    def_animated_icon!(Telescope);
    def_animated_icon!(TentTree);
    def_animated_icon!(Tent);
    def_animated_icon!(Terminal);
    def_animated_icon!(TestTubeDiagonal);
    def_animated_icon!(TestTube);
    def_animated_icon!(TestTubes);
    def_animated_icon!(TextAlignCenter);
    def_animated_icon!(TextAlignEnd);
    def_animated_icon!(TextAlignJustify);
    def_animated_icon!(TextAlignStart);
    def_animated_icon!(TextCursorInput);
    def_animated_icon!(TextCursor);
    def_animated_icon!(TextInitial);
    def_animated_icon!(TextQuote);
    def_animated_icon!(TextSearch);
    def_animated_icon!(TextWrap);
    def_animated_icon!(Theater);
    def_animated_icon!(ThermometerSnowflake);
    def_animated_icon!(ThermometerSun);
    def_animated_icon!(Thermometer);
    def_animated_icon!(ThumbsDown);
    def_animated_icon!(ThumbsUp);
    def_animated_icon!(TicketCheck);
    def_animated_icon!(TicketMinus);
    def_animated_icon!(TicketPercent);
    def_animated_icon!(TicketPlus);
    def_animated_icon!(TicketSlash);
    def_animated_icon!(TicketX);
    def_animated_icon!(Ticket);
    def_animated_icon!(TicketsPlane);
    def_animated_icon!(Tickets);
    def_animated_icon!(Timeline);
    def_animated_icon!(TimerOff);
    def_animated_icon!(TimerReset);
    def_animated_icon!(Timer);
    def_animated_icon!(ToggleLeft);
    def_animated_icon!(ToggleRight);
    def_animated_icon!(Toilet);
    def_animated_icon!(ToolCase);
    def_animated_icon!(Toolbox);
    def_animated_icon!(Tornado);
    def_animated_icon!(Torus);
    def_animated_icon!(TouchpadOff);
    def_animated_icon!(Touchpad);
    def_animated_icon!(TowelRack);
    def_animated_icon!(TowerControl);
    def_animated_icon!(ToyBrick);
    def_animated_icon!(Tractor);
    def_animated_icon!(TrafficCone);
    def_animated_icon!(TrainFrontTunnel);
    def_animated_icon!(TrainFront);
    def_animated_icon!(TrainTrack);
    def_animated_icon!(TramFront);
    def_animated_icon!(Transgender);
    def_animated_icon!(Trash2);
    def_animated_icon!(Trash);
    def_animated_icon!(TreeDeciduous);
    def_animated_icon!(TreePalm);
    def_animated_icon!(TreePine);
    def_animated_icon!(Trees);
    def_animated_icon!(TrendingDown);
    def_animated_icon!(TrendingUpDown);
    def_animated_icon!(TrendingUp);
    def_animated_icon!(TriangleAlert);
    def_animated_icon!(TriangleDashed);
    def_animated_icon!(TriangleRight);
    def_animated_icon!(Triangle);
    def_animated_icon!(Trophy);
    def_animated_icon!(TruckElectric);
    def_animated_icon!(Truck);
    def_animated_icon!(TurkishLira);
    def_animated_icon!(Turntable);
    def_animated_icon!(Turtle);
    def_animated_icon!(TvMinimalPlay);
    def_animated_icon!(TvMinimal);
    def_animated_icon!(Tv);
    def_animated_icon!(TypeOutline);
    def_animated_icon!(Type);
    def_animated_icon!(UmbrellaOff);
    def_animated_icon!(Umbrella);
    def_animated_icon!(Underline);
    def_animated_icon!(Undo2);
    def_animated_icon!(UndoDot);
    def_animated_icon!(Undo);
    def_animated_icon!(UnfoldHorizontal);
    def_animated_icon!(UnfoldVertical);
    def_animated_icon!(Ungroup);
    def_animated_icon!(University);
    def_animated_icon!(Unlink2);
    def_animated_icon!(Unlink);
    def_animated_icon!(Unplug);
    def_animated_icon!(Upload);
    def_animated_icon!(Usb);
    def_animated_icon!(UserCheck);
    def_animated_icon!(UserCog);
    def_animated_icon!(UserKey);
    def_animated_icon!(UserLock);
    def_animated_icon!(UserMinus);
    def_animated_icon!(UserPen);
    def_animated_icon!(UserPlus);
    def_animated_icon!(UserRoundArrowLeft);
    def_animated_icon!(UserRoundCheck);
    def_animated_icon!(UserRoundCog);
    def_animated_icon!(UserRoundKey);
    def_animated_icon!(UserRoundMinus);
    def_animated_icon!(UserRoundPen);
    def_animated_icon!(UserRoundPlus);
    def_animated_icon!(UserRoundSearch);
    def_animated_icon!(UserRoundX);
    def_animated_icon!(UserRound);
    def_animated_icon!(UserSearch);
    def_animated_icon!(UserStar);
    def_animated_icon!(UserX);
    def_animated_icon!(User);
    def_animated_icon!(UsersRound);
    def_animated_icon!(Users);
    def_animated_icon!(UtensilsCrossed);
    def_animated_icon!(Utensils);
    def_animated_icon!(UtilityPole);
    def_animated_icon!(Van);
    def_animated_icon!(Variable);
    def_animated_icon!(Vault);
    def_animated_icon!(VectorSquare);
    def_animated_icon!(Vegan);
    def_animated_icon!(VenetianMask);
    def_animated_icon!(VenusAndMars);
    def_animated_icon!(Venus);
    def_animated_icon!(VibrateOff);
    def_animated_icon!(Vibrate);
    def_animated_icon!(VideoOff);
    def_animated_icon!(Video);
    def_animated_icon!(Videotape);
    def_animated_icon!(View);
    def_animated_icon!(Voicemail);
    def_animated_icon!(Volleyball);
    def_animated_icon!(Volume1);
    def_animated_icon!(Volume2);
    def_animated_icon!(VolumeOff);
    def_animated_icon!(VolumeX);
    def_animated_icon!(Volume);
    def_animated_icon!(Vote);
    def_animated_icon!(WalletCards);
    def_animated_icon!(WalletMinimal);
    def_animated_icon!(Wallet);
    def_animated_icon!(Wallpaper);
    def_animated_icon!(WandSparkles);
    def_animated_icon!(Wand);
    def_animated_icon!(Warehouse);
    def_animated_icon!(WashingMachine);
    def_animated_icon!(Watch);
    def_animated_icon!(WavesArrowDown);
    def_animated_icon!(WavesArrowUp);
    def_animated_icon!(WavesHorizontal);
    def_animated_icon!(WavesLadder);
    def_animated_icon!(WavesVertical);
    def_animated_icon!(Waypoints);
    def_animated_icon!(WebcamOff);
    def_animated_icon!(Webcam);
    def_animated_icon!(WebhookOff);
    def_animated_icon!(Webhook);
    def_animated_icon!(WeightTilde);
    def_animated_icon!(Weight);
    def_animated_icon!(WheatOff);
    def_animated_icon!(Wheat);
    def_animated_icon!(WholeWord);
    def_animated_icon!(WifiCog);
    def_animated_icon!(WifiHigh);
    def_animated_icon!(WifiLow);
    def_animated_icon!(WifiOff);
    def_animated_icon!(WifiPen);
    def_animated_icon!(WifiSync);
    def_animated_icon!(WifiZero);
    def_animated_icon!(Wifi);
    def_animated_icon!(WindArrowDown);
    def_animated_icon!(Wind);
    def_animated_icon!(WineOff);
    def_animated_icon!(Wine);
    def_animated_icon!(Workflow);
    def_animated_icon!(Worm);
    def_animated_icon!(WrenchOff);
    def_animated_icon!(Wrench);
    def_animated_icon!(XLineTop);
    def_animated_icon!(X);
    def_animated_icon!(ZapOff);
    def_animated_icon!(Zap);
    def_animated_icon!(ZodiacAquarius);
    def_animated_icon!(ZodiacAries);
    def_animated_icon!(ZodiacCancer);
    def_animated_icon!(ZodiacCapricorn);
    def_animated_icon!(ZodiacGemini);
    def_animated_icon!(ZodiacLeo);
    def_animated_icon!(ZodiacLibra);
    def_animated_icon!(ZodiacOphiuchus);
    def_animated_icon!(ZodiacPisces);
    def_animated_icon!(ZodiacSagittarius);
    def_animated_icon!(ZodiacScorpio);
    def_animated_icon!(ZodiacTaurus);
    def_animated_icon!(ZodiacVirgo);
    def_animated_icon!(ZoomIn);
    def_animated_icon!(ZoomOut);
}
