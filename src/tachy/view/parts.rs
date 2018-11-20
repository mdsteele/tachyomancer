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

use cgmath::{Matrix4, Point2, vec3};
use num_integer::div_mod_floor;
use tachy::font::Align;
use tachy::gui::{Event, Resources};
use tachy::state::{ChipType, Rect, RectSize};

//===========================================================================//

const PART_WIDTH: i32 = 48;
const PART_HEIGHT: i32 = 48;
const PART_SPACING: i32 = 8;
const PART_COLUMNS: i32 = 3;
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
    parts: Vec<Part>,
    rect: Rect<i32>,
}

impl PartsTray {
    pub fn new(window_size: RectSize<u32>) -> PartsTray {
        // TODO: Change list of chip types based on puzzle type and game
        //   progress.
        let ctypes = vec![
            ChipType::Const(1),
            ChipType::Not,
            ChipType::And,
            ChipType::Pack,
            ChipType::Unpack,
            ChipType::Add,
            ChipType::Clock,
            ChipType::Delay,
            ChipType::Discard,
            ChipType::Join,
            ChipType::Latest,
            ChipType::Sample,
            ChipType::Ram,
            ChipType::Display,
            ChipType::Button,
        ];
        let parts = ctypes
            .into_iter()
            .enumerate()
            .map(|(index, ctype)| Part::new(ctype, index as i32))
            .collect::<Vec<Part>>();
        let rect = Rect::new(0,
                             TRAY_OUTER_MARGIN,
                             TRAY_WIDTH,
                             (window_size.height as i32) -
                                 2 * TRAY_OUTER_MARGIN);
        PartsTray { parts, rect }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let x = self.rect.x() as f32;
        let y = self.rect.y() as f32;
        let w = self.rect.width() as f32;
        let h = self.rect.height() as f32;
        resources
            .shaders()
            .solid()
            .fill_rect(matrix, (0.0, 0.5, 0.0), (x, y, w, h));
        let matrix = matrix * Matrix4::from_translation(vec3(x, y, 0.0));
        for part in self.parts.iter() {
            part.draw(resources, &matrix);
        }
    }

    pub fn handle_event(&mut self, event: &Event)
                        -> (Option<PartsAction>, bool) {
        match event {
            Event::MouseDown(mouse) if self.rect.contains_point(mouse.pt) => {
                let delta = self.rect.top_left() - Point2::new(0, 0);
                for part in self.parts.iter() {
                    if part.rect.contains_point(mouse.pt - delta) {
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
            _ => (None, false),
        }
    }
}

//===========================================================================//

struct Part {
    rect: Rect<i32>,
    ctype: ChipType,
}

impl Part {
    fn new(ctype: ChipType, index: i32) -> Part {
        let (row, col) = div_mod_floor(index, PART_COLUMNS);
        let rect =
            Rect::new(TRAY_INNER_MARGIN + col * (PART_WIDTH + PART_SPACING),
                      TRAY_INNER_MARGIN + row * (PART_HEIGHT + PART_SPACING),
                      PART_WIDTH,
                      PART_HEIGHT);
        Part { rect, ctype }
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let x = self.rect.x() as f32;
        let y = self.rect.y() as f32;
        let w = self.rect.width() as f32;
        let h = self.rect.height() as f32;
        resources
            .shaders()
            .solid()
            .fill_rect(matrix, (0.75, 0.0, 0.0), (x, y, w, h));
        resources.fonts().roman().draw(matrix,
                                       (10.0, 20.0),
                                       Align::Center,
                                       (x + 0.5 * w, y + 0.5 * h - 10.0),
                                       &format!("{:?}", self.ctype));
    }
}

//===========================================================================//
