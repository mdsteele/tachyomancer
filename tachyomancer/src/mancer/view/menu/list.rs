// +--------------------------------------------------------------------------+
// | Copyright 2018 Matthew D. Steele <mdsteele@alum.mit.edu>                 |
// |                                                                          |
// | This file is part of Tachyomancer.                                       |
// |                                                                          |
// | Tachyomancer is free software: you can redistribute it and/or modify it  |
// | under the terms of the GNU General Public License as published by the    |
// | Free Software Foundation, either version 3 of the License, or (at your   |
// | option) any later version.                                               |
// |                                                                          |
// | Tachyomancer is distributed in the hope that it will be useful, but      |
// | WITHOUT ANY WARRANTY; without even the implied warranty of               |
// | MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU        |
// | General Public License for details.                                      |
// |                                                                          |
// | You should have received a copy of the GNU General Public License along  |
// | with Tachyomancer.  If not, see <http://www.gnu.org/licenses/>.          |
// +--------------------------------------------------------------------------+

use super::super::button::Scrollbar;
use crate::mancer::font::Align;
use crate::mancer::gl::Stencil;
use crate::mancer::gui::{Event, Resources, Sound, Ui};
use cgmath::Matrix4;
use num_integer::div_mod_floor;
use std::borrow::Borrow;
use tachy::geom::{AsFloat, Color3, Color4, Rect};

//===========================================================================//

const FONT_SIZE: f32 = 20.0;

const ICON_WIDTH: i32 = 32;
const ICON_HEIGHT: i32 = 32;

const ITEM_HEIGHT: i32 = 50;
const ITEM_SPACING: i32 = -1;
const ITEM_INNER_MARGIN: i32 = 10;

const SCROLLBAR_WIDTH: i32 = 18;
const SCROLLBAR_MARGIN: i32 = 3;

pub const fn list_height_for_num_items(num_items: i32) -> i32 {
    num_items * (ITEM_HEIGHT + ITEM_SPACING) - ITEM_SPACING
}

//===========================================================================//

// Generated code:
// enum ListIcon { ... }
include!(concat!(env!("OUT_DIR"), "/texture/list_icons.rs"));

//===========================================================================//

pub struct ListView<T> {
    rect: Rect<i32>,
    items: Vec<(T, String, bool, Option<ListIcon>)>,
    scrollbar: Scrollbar,
}

impl<T: Clone + Eq> ListView<T> {
    pub fn new<Q>(
        rect: Rect<i32>,
        ui: &mut Ui,
        items: Vec<(T, String, bool, Option<ListIcon>)>,
        current: &Q,
    ) -> ListView<T>
    where
        Q: PartialEq + ?Sized,
        T: Borrow<Q>,
    {
        let scrollbar_rect = Rect::new(
            rect.right() - SCROLLBAR_WIDTH,
            rect.y,
            SCROLLBAR_WIDTH,
            rect.height,
        );
        let mut list = ListView {
            rect,
            items: Vec::new(),
            scrollbar: Scrollbar::new(scrollbar_rect, 0),
        };
        list.set_items(ui, items, current);
        list
    }

    pub fn draw<Q>(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        current: &Q,
    ) where
        Q: PartialEq + ?Sized,
        T: Borrow<Q>,
    {
        resources.shaders().shadow().rect_shadow_basic(
            matrix,
            self.rect.as_f32(),
            Color3::CYAN1,
        );
        {
            let stencil = Stencil::new();
            self.draw_background(resources, matrix);
            stencil.enable_clipping();
            self.draw_items(resources, matrix, current);
        }
        self.draw_frame(resources, matrix);
        self.scrollbar.draw(resources, matrix);
    }

    fn draw_background(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let mut rect = self.rect.as_f32();
        rect.width = self.item_width() as f32;
        rect = rect.expand(-2.0);
        let solid = resources.shaders().solid();
        solid.tint_rect(matrix, Color4::PURPLE0_TRANSLUCENT, rect);
    }

    fn draw_items<Q>(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        current: &Q,
    ) where
        Q: PartialEq + ?Sized,
        T: Borrow<Q>,
    {
        let item_width = self.item_width() as f32;
        let ui = resources.shaders().ui();
        for (index, &(ref value, ref label, bold, opt_icon)) in
            self.items.iter().enumerate()
        {
            // Item frame:
            let top = self.rect.y
                + (index as i32) * (ITEM_HEIGHT + ITEM_SPACING)
                - self.scrollbar.scroll_top();
            if top >= self.rect.bottom() || top + ITEM_HEIGHT <= self.rect.y {
                continue;
            }
            let bg_color = if value.borrow() == current {
                Color3::PURPLE4.with_alpha(0.5)
            } else {
                Color4::TRANSPARENT
            };
            let rect = Rect::new(
                self.rect.x as f32,
                top as f32,
                item_width,
                ITEM_HEIGHT as f32,
            );
            ui.draw_list_item(
                matrix,
                &rect,
                &Color4::CYAN2,
                &Color4::ORANGE2,
                &bg_color,
            );

            // Icon:
            if let Some(icon) = opt_icon {
                let icon_rect = Rect::new(
                    rect.x + (ITEM_INNER_MARGIN as f32),
                    rect.y + 0.5 * (ITEM_HEIGHT - ICON_HEIGHT) as f32,
                    ICON_WIDTH as f32,
                    ICON_HEIGHT as f32,
                );
                let color =
                    if bold { &Color4::ORANGE3 } else { &Color4::ORANGE5 };
                let texture = resources.textures().list_icons();
                let shader = resources.shaders().icon();
                shader.draw(matrix, icon_rect, icon as u32, color, texture);
            }

            // Label:
            let text_offset = if opt_icon.is_some() {
                ICON_WIDTH + ITEM_INNER_MARGIN
            } else {
                0
            };
            let text_left =
                (self.rect.x + ITEM_INNER_MARGIN + text_offset) as f32;
            let text_top = (top + ITEM_HEIGHT / 2) as f32;
            let (font, text_color) = if bold {
                (resources.fonts().bold(), &Color4::ORANGE4)
            } else {
                (resources.fonts().roman(), &Color4::WHITE)
            };
            font.draw_style(
                matrix,
                FONT_SIZE,
                Align::MidLeft,
                (text_left, text_top),
                text_color,
                0.0,
                label.as_str(),
            );
        }
    }

    fn draw_frame(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let mut rect = self.rect.as_f32();
        rect.width = self.item_width() as f32;
        let ui = resources.shaders().ui();
        ui.draw_list_frame(
            matrix,
            &rect,
            &Color4::CYAN2,
            &Color4::ORANGE2,
            &Color4::PURPLE0,
        );
    }

    pub fn on_event<Q>(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        current: &Q,
    ) -> Option<T>
    where
        Q: PartialEq + ?Sized,
        T: Borrow<Q>,
    {
        self.scrollbar.on_event(event, ui);
        match event {
            Event::MouseDown(mouse)
                if mouse.left && self.rect.contains_point(mouse.pt) =>
            {
                if mouse.pt.x - self.rect.x < self.item_width() {
                    let (index, rel_y) = div_mod_floor(
                        mouse.pt.y - self.rect.y + self.scrollbar.scroll_top(),
                        ITEM_HEIGHT + ITEM_SPACING,
                    );
                    if rel_y < ITEM_HEIGHT
                        && index >= 0
                        && (index as usize) < self.items.len()
                    {
                        let value = &self.items[index as usize].0;
                        if value.borrow() != current {
                            ui.audio().play_sound(Sound::ButtonClick);
                            return Some(value.clone());
                        }
                    }
                }
            }
            Event::Scroll(scroll) if self.rect.contains_point(scroll.pt) => {
                self.scrollbar.scroll_by(scroll.delta.y, ui);
            }
            _ => {}
        }
        return None;
    }

    pub fn set_items<Q>(
        &mut self,
        ui: &mut Ui,
        items: Vec<(T, String, bool, Option<ListIcon>)>,
        current: &Q,
    ) where
        Q: PartialEq + ?Sized,
        T: Borrow<Q>,
    {
        let num_items = items.len() as i32;
        let total_height = list_height_for_num_items(num_items);
        self.scrollbar.set_total_height(total_height, ui);
        let current_index = items
            .iter()
            .position(|(value, _, _, _)| value.borrow() == current)
            .unwrap_or(0);
        let mid_current = (current_index as i32)
            * (ITEM_HEIGHT + ITEM_SPACING)
            + ITEM_HEIGHT / 2;
        self.scrollbar.scroll_to(mid_current, ui);
        self.items = items;
        ui.request_redraw();
    }

    fn item_width(&self) -> i32 {
        if self.scrollbar.is_visible() {
            self.rect.width - (SCROLLBAR_MARGIN + SCROLLBAR_WIDTH)
        } else {
            self.rect.width
        }
    }
}

//===========================================================================//
