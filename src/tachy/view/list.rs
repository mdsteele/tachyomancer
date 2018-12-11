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

use cgmath::Matrix4;
use num_integer::div_mod_floor;
use std::borrow::Borrow;
use tachy::font::Align;
use tachy::gl::Stencil;
use tachy::gui::{Event, Resources};
use tachy::state::Rect;

//===========================================================================//

const FONT_SIZE: f32 = 20.0;

const ITEM_HEIGHT: i32 = 50;
const ITEM_SPACING: i32 = 10;
const ITEM_INNER_MARGIN: i32 = 10;

const SCROLLBAR_WIDTH: i32 = 15;
const SCROLLBAR_MARGIN: i32 = 5;

//===========================================================================//

pub struct ListView<T> {
    rect: Rect<i32>,
    items: Vec<(T, String)>, // TODO: icons
    scroll_top: i32,
    scroll_max: i32,
    drag: Option<i32>,
}

impl<T: Clone + Eq> ListView<T> {
    pub fn new<Q>(rect: Rect<i32>, current: &Q, items: Vec<(T, String)>)
                  -> ListView<T>
    where
        Q: PartialEq + ?Sized,
        T: Borrow<Q>,
    {
        let mut list = ListView {
            rect,
            items: Vec::new(),
            scroll_top: 0,
            scroll_max: 0,
            drag: None,
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
        // Draw background and define clipping area:
        let stencil = Stencil::new();
        {
            let color = (0.1, 0.1, 0.1);
            let rect = (self.rect.x as f32,
                        self.rect.y as f32,
                        self.rect.width as f32,
                        self.rect.height as f32);
            resources.shaders().solid().fill_rect(&matrix, color, rect);
        }
        stencil.enable_clipping();

        // Draw list items:
        let item_width = self.item_width();
        for (index, &(ref value, ref label)) in self.items.iter().enumerate() {
            let top = self.rect.y +
                (index as i32) * (ITEM_HEIGHT + ITEM_SPACING) -
                self.scroll_top;
            if top >= self.rect.bottom() || top + ITEM_HEIGHT <= self.rect.y {
                continue;
            }
            let color = if value.borrow() == current {
                (0.6, 0.1, 0.1)
            } else {
                (0.1, 0.1, 0.6)
            };
            let rect = (self.rect.x as f32,
                        top as f32,
                        item_width as f32,
                        ITEM_HEIGHT as f32);
            resources.shaders().solid().fill_rect(&matrix, color, rect);
            resources.fonts().roman().draw(&matrix,
                                           FONT_SIZE,
                                           Align::Left,
                                           ((self.rect.x +
                                                ITEM_INNER_MARGIN) as
                                                f32,
                                            (top + ITEM_HEIGHT / 2) as f32 -
                                                0.5 * FONT_SIZE),
                                           label.as_str());
        }

        // Draw scrollbar:
        if let Some(handle_rect) = self.scroll_handle_rect() {
            let color = (0.3, 0.1, 0.3);
            let rect = ((self.rect.right() - SCROLLBAR_WIDTH) as f32,
                        self.rect.y as f32,
                        SCROLLBAR_WIDTH as f32,
                        self.rect.height as f32);
            resources.shaders().solid().fill_rect(&matrix, color, rect);
            let color = if self.drag.is_some() {
                (0.9, 0.6, 0.9)
            } else {
                (0.9, 0.1, 0.9)
            };
            let rect = (handle_rect.x as f32,
                        handle_rect.y as f32,
                        handle_rect.width as f32,
                        handle_rect.height as f32);
            resources.shaders().solid().fill_rect(&matrix, color, rect);
        }
    }

    pub fn handle_event<Q>(&mut self, event: &Event, current: &Q) -> Option<T>
    where
        Q: PartialEq + ?Sized,
        T: Borrow<Q>,
    {
        match event {
            Event::MouseDown(mouse)
                if mouse.left && self.rect.contains_point(mouse.pt) => {
                if let Some(handle_rect) = self.scroll_handle_rect() {
                    if handle_rect.contains_point(mouse.pt) {
                        self.drag = Some(mouse.pt.y - handle_rect.y);
                    }
                    // TODO: support jumping up/down page
                }
                if mouse.pt.x - self.rect.x < self.item_width() {
                    let (index, rel_y) =
                        div_mod_floor(mouse.pt.y - self.rect.y +
                                          self.scroll_top,
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
            Event::MouseMove(mouse) => {
                if let Some(drag_offset) = self.drag {
                    let new_handle_y = mouse.pt.y - drag_offset - self.rect.y;
                    let total_height = self.scroll_max + self.rect.height;
                    let new_scroll_top = div_round(total_height *
                                                       new_handle_y,
                                                   self.rect.height);
                    self.scroll_top =
                        new_scroll_top.max(0).min(self.scroll_max);
                }
            }
            Event::MouseUp(mouse) if mouse.left => {
                self.drag = None;
            }
            // TODO: support mouse wheel and two-finger scrolling
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
        self.scroll_max = (total_height - self.rect.height).max(0);
        let current_index = items
            .iter()
            .position(|(value, _)| value.borrow() == current)
            .unwrap_or(0);
        let mid_current = (current_index as i32) *
            (ITEM_HEIGHT + ITEM_SPACING) +
            ITEM_HEIGHT / 2;
        self.scroll_top =
            (mid_current - self.rect.height / 2).max(0).min(self.scroll_max);
        self.items = items;
    }

    pub fn unfocus(&mut self) { self.drag = None; }

    fn has_scrollbar(&self) -> bool { self.scroll_max != 0 }

    fn scroll_handle_rect(&self) -> Option<Rect<i32>> {
        if self.scroll_max != 0 {
            let total_height = self.scroll_max + self.rect.height;
            Some(Rect::new(self.rect.right() - SCROLLBAR_WIDTH,
                           self.rect.y +
                               div_round(self.rect.height * self.scroll_top,
                                         total_height),
                           SCROLLBAR_WIDTH,
                           div_round(self.rect.height * self.rect.height,
                                     total_height)))
        } else {
            None
        }
    }

    fn item_width(&self) -> i32 {
        if self.has_scrollbar() {
            self.rect.width - (SCROLLBAR_MARGIN + SCROLLBAR_WIDTH)
        } else {
            self.rect.width
        }
    }
}

//===========================================================================//

fn div_round(a: i32, b: i32) -> i32 {
    ((a as f64) / (b as f64)).round() as i32
}

//===========================================================================//
