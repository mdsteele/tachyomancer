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
use std::collections::HashSet;
use std::u32;
use tachy::font::Align;
use tachy::geom::{Color4, Rect, RectSize};
use tachy::gui::{Event, Resources};
use tachy::save::{Puzzle, PuzzleKind};
use tachy::state::EvalError;

//===========================================================================//

const TRAY_INNER_MARGIN: i32 = 20;
const TRAY_OUTER_MARGIN: i32 = 80;
const TRAY_WIDTH: i32 = 200;

const TABLE_ROW_HEIGHT: i32 = 20;
const TABLE_FONT_SIZE: f32 = 16.0;

//===========================================================================//

pub struct VerificationTray {
    rect: Rect<i32>,
    subview: Box<PuzzleVerifyView>,
}

impl VerificationTray {
    pub fn new(window_size: RectSize<i32>, current_puzzle: Puzzle)
               -> VerificationTray {
        let rect = if current_puzzle.kind() == PuzzleKind::Sandbox {
            Rect::new(window_size.width, 0, 0, 0)
        } else {
            Rect::new(window_size.width - TRAY_WIDTH,
                      TRAY_OUTER_MARGIN,
                      TRAY_WIDTH,
                      window_size.height - 2 * TRAY_OUTER_MARGIN)
        };
        let inner_rect = Rect::new(rect.x + TRAY_INNER_MARGIN,
                                   rect.y + TRAY_INNER_MARGIN,
                                   rect.width - 2 * TRAY_INNER_MARGIN,
                                   rect.height - 2 * TRAY_INNER_MARGIN);
        let subview =
            match current_puzzle {
                Puzzle::AutomateHeliostat => HeliostatCam::new(inner_rect),
                _ => {
                    FabricationTable::new(
                        inner_rect,
                        vec![
                            "in1".to_string(),
                            "in2".to_string(),
                            "out".to_string(),
                        ],
                    )
                }
            };
        VerificationTray { rect, subview }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                time_step: Option<u32>, puzzle_data: &[u64],
                errors: &[EvalError]) {
        if self.rect.is_empty() {
            return;
        }
        let ui = resources.shaders().ui();
        ui.draw_box2(matrix,
                     &self.rect.as_f32(),
                     &Color4::ORANGE2,
                     &Color4::CYAN2,
                     &Color4::PURPLE0.with_alpha(0.8));
        self.subview.draw(resources, matrix, time_step, puzzle_data, errors);
    }

    pub fn on_event(&mut self, event: &Event) -> bool {
        match event {
            Event::MouseDown(mouse) if self.rect.contains_point(mouse.pt) => {
                true
            }
            Event::Scroll(scroll) if self.rect.contains_point(scroll.pt) => {
                true
            }
            _ => false,
        }
    }
}

//===========================================================================//

trait PuzzleVerifyView {
    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
            time_step: Option<u32>, values: &[u64], errors: &[EvalError]);
}

//===========================================================================//

struct FabricationTable {
    rect: Rect<i32>,
    columns: Vec<String>,
}

impl FabricationTable {
    fn new(rect: Rect<i32>, columns: Vec<String>) -> Box<PuzzleVerifyView> {
        Box::new(FabricationTable { rect, columns })
    }
}

impl PuzzleVerifyView for FabricationTable {
    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
            time_step: Option<u32>, values: &[u64], errors: &[EvalError]) {
        let rect = self.rect.as_f32();
        let column_width = rect.width / (self.columns.len() as f32);
        let row_height = TABLE_ROW_HEIGHT as f32;
        for (index, column_name) in self.columns.iter().enumerate() {
            let font = resources.fonts().roman();
            font.draw(matrix,
                      TABLE_FONT_SIZE,
                      Align::MidCenter,
                      (rect.x + ((index as f32) + 0.5) * column_width,
                       rect.y + 0.5 * row_height),
                      &column_name);
        }
        let num_columns = self.columns.len();
        let num_rows = values.len() / num_columns;

        let mut error_rows = HashSet::<usize>::new();
        for error in errors.iter() {
            error_rows.insert(error.time_step as usize);
        }

        for row in 0..num_rows {
            let color = if Some(row as u32) == time_step {
                (0.7, 0.7, 1.0)
            } else if error_rows.contains(&row) {
                (0.7, 0.3, 0.3)
            } else {
                (0.1, 0.1, 0.1)
            };
            let row_top = rect.y + ((row + 1) as f32) * row_height;
            let rect = Rect::new(rect.x, row_top, rect.width, row_height);
            resources.shaders().solid().fill_rect(matrix, color, rect);
            let row_center = row_top + 0.5 * row_height;
            for col in 0..num_columns {
                let value = values[row * num_columns + col];
                if value > u32::MAX as u64 {
                    continue;
                }
                let col_center = rect.x + ((col as f32) + 0.5) * column_width;
                let font = resources.fonts().roman();
                font.draw(matrix,
                          TABLE_FONT_SIZE,
                          Align::MidCenter,
                          (col_center, row_center),
                          &value.to_string());
            }
        }
    }
}

//===========================================================================//

struct HeliostatCam {
    rect: Rect<i32>,
}

impl HeliostatCam {
    fn new(rect: Rect<i32>) -> Box<PuzzleVerifyView> {
        Box::new(HeliostatCam { rect })
    }
}

impl PuzzleVerifyView for HeliostatCam {
    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
            _time_step: Option<u32>, values: &[u64], _errors: &[EvalError]) {
        debug_assert_eq!(values.len(), 5);
        // TODO: Draw a grid view of the heliostat coordinates, and a visual
        //   energy meter.
        let left = self.rect.x as f32;
        let top = self.rect.y as f32;
        let font = resources.fonts().roman();
        font.draw(matrix,
                  TABLE_FONT_SIZE,
                  Align::TopLeft,
                  (left, top),
                  &format!("Energy: {}", values[4]));
        font.draw(matrix,
                  TABLE_FONT_SIZE,
                  Align::TopLeft,
                  (left, top + 30.0),
                  &format!("Pos: ({}, {})", values[2], values[3]));
        font.draw(matrix,
                  TABLE_FONT_SIZE,
                  Align::TopLeft,
                  (left, top + 60.0),
                  &format!("Opt: ({}, {})", values[0], values[1]));
    }
}

//===========================================================================//
