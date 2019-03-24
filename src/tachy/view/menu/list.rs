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
use cgmath::Matrix4;
use num_integer::div_mod_floor;
use std::borrow::Borrow;
use tachy::font::Align;
use tachy::geom::{AsFloat, Color4, Rect};
use tachy::gl::Stencil;
use tachy::gui::{Event, Resources};

//===========================================================================//

const FONT_SIZE: f32 = 20.0;

const ITEM_HEIGHT: i32 = 50;
const ITEM_SPACING: i32 = -1;
const ITEM_INNER_MARGIN: i32 = 10;

const SCROLLBAR_WIDTH: i32 = 18;
const SCROLLBAR_MARGIN: i32 = 3;

//===========================================================================//

pub struct ListView<T> {
    rect: Rect<i32>,
    items: Vec<(T, String)>, // TODO: icons
    scrollbar: Scrollbar,
}

impl<T: Clone + Eq> ListView<T> {
    pub fn new<Q>(rect: Rect<i32>, current: &Q, items: Vec<(T, String)>)
                  -> ListView<T>
    where
        Q: PartialEq + ?Sized,
        T: Borrow<Q>,
    {
        let scrollbar_rect = Rect::new(rect.right() - SCROLLBAR_WIDTH,
                                       rect.y,
                                       SCROLLBAR_WIDTH,
                                       rect.height);
        let mut list = ListView {
            rect,
            items: Vec::new(),
            scrollbar: Scrollbar::new(scrollbar_rect),
        };
        list.set_items(current, items);
        list
    }

    pub fn draw<Q>(&self, resources: &Resources, matrix: &Matrix4<f32>,
                   current: &Q)
    where
        Q: PartialEq + ?Sized,
        T: Borrow<Q>,
    {
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
        solid.fill_rect(matrix, Color4::PURPLE0.rgb(), rect);
    }

    fn draw_items<Q>(&self, resources: &Resources, matrix: &Matrix4<f32>,
                     current: &Q)
    where
        Q: PartialEq + ?Sized,
        T: Borrow<Q>,
    {
        let item_width = self.item_width() as f32;
        let ui = resources.shaders().ui();
        for (index, &(ref value, ref label)) in self.items.iter().enumerate() {
            let top = self.rect.y +
                (index as i32) * (ITEM_HEIGHT + ITEM_SPACING) -
                self.scrollbar.scroll_top();
            if top >= self.rect.bottom() || top + ITEM_HEIGHT <= self.rect.y {
                continue;
            }
            let bg_color = if value.borrow() == current {
                Color4::PURPLE3
            } else {
                Color4::PURPLE0
            };
            let rect = Rect::new(self.rect.x as f32,
                                 top as f32,
                                 item_width,
                                 ITEM_HEIGHT as f32);
            ui.draw_list_item(matrix,
                              &rect,
                              &Color4::CYAN2,
                              &Color4::ORANGE2,
                              &bg_color);
            let font = resources.fonts().roman();
            font.draw(matrix,
                      FONT_SIZE,
                      Align::MidLeft,
                      ((self.rect.x + ITEM_INNER_MARGIN) as f32,
                       (top + ITEM_HEIGHT / 2) as f32),
                      label.as_str());
        }
    }

    fn draw_frame(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let mut rect = self.rect.as_f32();
        rect.width = self.item_width() as f32;
        let ui = resources.shaders().ui();
        ui.draw_list_frame(matrix,
                           &rect,
                           &Color4::CYAN2,
                           &Color4::ORANGE2,
                           &Color4::PURPLE0);
    }

    pub fn on_event<Q>(&mut self, event: &Event, current: &Q) -> Option<T>
    where
        Q: PartialEq + ?Sized,
        T: Borrow<Q>,
    {
        self.scrollbar.on_event(event);
        match event {
            Event::MouseDown(mouse)
                if mouse.left && self.rect.contains_point(mouse.pt) => {
                if mouse.pt.x - self.rect.x < self.item_width() {
                    let (index, rel_y) =
                        div_mod_floor(mouse.pt.y - self.rect.y +
                                          self.scrollbar.scroll_top(),
                                      ITEM_HEIGHT + ITEM_SPACING);
                    if rel_y < ITEM_HEIGHT && index >= 0 &&
                        (index as usize) < self.items.len()
                    {
                        let value = &self.items[index as usize].0;
                        if value.borrow() != current {
                            // TODO: Play sound
                            return Some(value.clone());
                        }
                    }
                }
            }
            Event::Scroll(scroll) if self.rect.contains_point(scroll.pt) => {
                self.scrollbar.scroll_by(scroll.delta.y);
            }
            _ => {}
        }
        return None;
    }

    pub fn set_items<Q>(&mut self, current: &Q, items: Vec<(T, String)>)
    where
        Q: PartialEq + ?Sized,
        T: Borrow<Q>,
    {
        let num_items = items.len() as i32;
        let total_height = num_items * (ITEM_HEIGHT + ITEM_SPACING) -
            ITEM_SPACING;
        self.scrollbar.set_total_height(total_height);
        let current_index = items
            .iter()
            .position(|(value, _)| value.borrow() == current)
            .unwrap_or(0);
        let mid_current = (current_index as i32) *
            (ITEM_HEIGHT + ITEM_SPACING) +
            ITEM_HEIGHT / 2;
        self.scrollbar.scroll_to(mid_current);
        self.items = items;
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
