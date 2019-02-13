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

use super::button::Scrollbar;
use cgmath::Matrix4;
use num_integer::div_mod_floor;
use std::borrow::Borrow;
use tachy::font::Align;
use tachy::geom::Rect;
use tachy::gl::Stencil;
use tachy::gui::{Event, Resources};

//===========================================================================//

const FONT_SIZE: f32 = 20.0;

const ITEM_HEIGHT: i32 = 50;
const ITEM_SPACING: i32 = 10;
const ITEM_INNER_MARGIN: i32 = 10;

const SCROLLBAR_WIDTH: i32 = 18;
const SCROLLBAR_MARGIN: i32 = 5;

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
        // Draw background and define clipping area:
        let stencil = Stencil::new();
        {
            let color = (0.1, 0.1, 0.1);
            resources
                .shaders()
                .solid()
                .fill_rect(matrix, color, self.rect.as_f32());
        }
        stencil.enable_clipping();

        // Draw list items:
        let item_width = self.item_width();
        for (index, &(ref value, ref label)) in self.items.iter().enumerate() {
            let top = self.rect.y +
                (index as i32) * (ITEM_HEIGHT + ITEM_SPACING) -
                self.scrollbar.scroll_top();
            if top >= self.rect.bottom() || top + ITEM_HEIGHT <= self.rect.y {
                continue;
            }
            let color = if value.borrow() == current {
                (0.6, 0.1, 0.1)
            } else {
                (0.1, 0.1, 0.6)
            };
            let rect = Rect::new(self.rect.x as f32,
                                 top as f32,
                                 item_width as f32,
                                 ITEM_HEIGHT as f32);
            resources.shaders().solid().fill_rect(matrix, color, rect);
            let font = resources.fonts().roman();
            font.draw(matrix,
                      FONT_SIZE,
                      Align::MidLeft,
                      ((self.rect.x + ITEM_INNER_MARGIN) as f32,
                       (top + ITEM_HEIGHT / 2) as f32),
                      label.as_str());
        }

        // Draw scrollbar:
        self.scrollbar.draw(resources, matrix);
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
