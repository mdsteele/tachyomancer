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

use cgmath::{Matrix4, Point2};
use tachy::font::Align;
use tachy::geom::{AsFloat, Color4, Rect, RectSize};
use tachy::gui::{Event, Resources};
use tachy::save::{CHIP_CATEGORIES, ChipType, Puzzle};

//===========================================================================//

const CATEGORY_LABEL_HEIGHT: i32 = 30;
const CATEGORY_LABEL_FONT_SIZE: f32 = 20.0;

const PART_WIDTH: i32 = 48;
const PART_HEIGHT: i32 = 48;
const PART_SPACING: i32 = 8;
const PART_COLUMNS: i32 = 4;

const TRAY_INNER_MARGIN: i32 = 16;
const TRAY_OUTER_MARGIN: i32 = 32;
const TRAY_WIDTH: i32 = 2 * TRAY_INNER_MARGIN +
    PART_COLUMNS * (PART_WIDTH + PART_SPACING) -
    PART_SPACING;

//===========================================================================//

#[derive(Clone, Copy, Debug)]
pub enum PartsAction {
    Grab(ChipType, Point2<i32>),
    Drop,
}

//===========================================================================//

pub struct PartsTray {
    rect: Rect<i32>,
    category_labels: Vec<CategoryLabel>,
    parts: Vec<Part>,
}

impl PartsTray {
    pub fn new(window_size: RectSize<i32>, current_puzzle: Puzzle)
               -> PartsTray {
        let rect = Rect::new(0,
                             TRAY_OUTER_MARGIN,
                             TRAY_WIDTH,
                             window_size.height - 2 * TRAY_OUTER_MARGIN);

        let mut num_parts: usize = 0;
        let mut categories = Vec::<(&str, Vec<ChipType>)>::new();
        for &(name, ctypes) in CHIP_CATEGORIES.iter() {
            let allowed_ctypes: Vec<ChipType> = ctypes
                .iter()
                .cloned()
                .filter(|ctype| ctype.is_allowed_in(current_puzzle))
                .collect();
            if !allowed_ctypes.is_empty() {
                num_parts += allowed_ctypes.len();
                categories.push((name, allowed_ctypes));
            }
        }

        let mut category_labels =
            Vec::<CategoryLabel>::with_capacity(categories.len());
        let mut parts = Vec::<Part>::with_capacity(num_parts);
        let mut top = rect.y + TRAY_INNER_MARGIN;
        for (name, ctypes) in categories {
            category_labels.push(CategoryLabel::new(top, name));
            top += CATEGORY_LABEL_HEIGHT;
            let mut col = 0;
            for ctype in ctypes {
                let left = TRAY_INNER_MARGIN +
                    col * (PART_WIDTH + PART_SPACING);
                parts.push(Part::new(left, top, ctype));
                col += 1;
                if col >= PART_COLUMNS {
                    col = 0;
                    top += PART_HEIGHT + PART_SPACING;
                }
            }
            if col > 0 {
                top += PART_HEIGHT + PART_SPACING;
            }
        }

        PartsTray {
            rect,
            category_labels,
            parts,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let ui = resources.shaders().ui();
        let rect = self.rect.as_f32();
        ui.draw_box2(matrix,
                     &rect,
                     &Color4::ORANGE2,
                     &Color4::CYAN2,
                     &Color4::PURPLE0.with_alpha(0.8));
        for label in self.category_labels.iter() {
            label.draw(resources, matrix);
        }
        for part in self.parts.iter() {
            part.draw(resources, &matrix);
        }
    }

    pub fn on_event(&mut self, event: &Event) -> (Option<PartsAction>, bool) {
        match event {
            Event::MouseDown(mouse) if self.rect.contains_point(mouse.pt) => {
                for part in self.parts.iter() {
                    if part.rect.contains_point(mouse.pt) {
                        let action = PartsAction::Grab(part.ctype, mouse.pt);
                        return (Some(action), true);
                    }
                }
                (None, true)
            }
            Event::MouseUp(mouse) => {
                if mouse.left && self.rect.contains_point(mouse.pt) {
                    (Some(PartsAction::Drop), false)
                } else {
                    (None, false)
                }
            }
            Event::Scroll(scroll) if self.rect.contains_point(scroll.pt) => {
                (None, true)
            }
            _ => (None, false),
        }
    }
}

//===========================================================================//

struct CategoryLabel {
    top: i32,
    text: &'static str,
}

impl CategoryLabel {
    fn new(top: i32, text: &'static str) -> CategoryLabel {
        CategoryLabel { top, text }
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        resources.fonts().roman().draw(matrix,
                                       CATEGORY_LABEL_FONT_SIZE,
                                       Align::TopLeft,
                                       (TRAY_INNER_MARGIN as f32,
                                        self.top as f32),
                                       self.text);
    }
}

//===========================================================================//

struct Part {
    rect: Rect<i32>,
    ctype: ChipType,
}

impl Part {
    fn new(left: i32, top: i32, ctype: ChipType) -> Part {
        let rect = Rect::new(left, top, PART_WIDTH, PART_HEIGHT);
        Part { rect, ctype }
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(matrix, (0.75, 0.0, 0.0), rect);
        resources.fonts().roman().draw(matrix,
                                       20.0,
                                       Align::MidCenter,
                                       (rect.x + 0.5 * rect.width,
                                        rect.y + 0.5 * rect.height),
                                       &format!("{:?}", self.ctype));
    }
}

//===========================================================================//
