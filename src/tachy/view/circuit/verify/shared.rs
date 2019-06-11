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
use std::collections::HashSet;
use std::marker::PhantomData;
use std::u32;
use tachy::font::Align;
use tachy::geom::{AsFloat, Color3, Rect, RectSize};
use tachy::gui::Resources;
use tachy::state::{CircuitEval, EvalError, FabricationEval};

//===========================================================================//

pub trait PuzzleVerifyView {
    fn size(&self) -> RectSize<i32>;

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
            circuit_eval: Option<&CircuitEval>);
}

//===========================================================================//

pub struct NullVerifyView {}

impl NullVerifyView {
    pub fn new() -> Box<PuzzleVerifyView> { Box::new(NullVerifyView {}) }
}

impl PuzzleVerifyView for NullVerifyView {
    fn size(&self) -> RectSize<i32> { RectSize::new(0, 0) }

    fn draw(&self, _resources: &Resources, _matrix: &Matrix4<f32>,
            _circuit_eval: Option<&CircuitEval>) {
    }
}

//===========================================================================//

pub struct FabricationVerifyView<T> {
    table: FabricationTable,
    phantom: PhantomData<T>,
}

impl<T: FabricationEval> FabricationVerifyView<T> {
    pub fn new(right_bottom: Point2<i32>) -> Box<PuzzleVerifyView> {
        let table = FabricationTable::new(right_bottom,
                                          T::table_column_names(),
                                          T::expected_table_values());
        Box::new(FabricationVerifyView::<T> {
                     table,
                     phantom: PhantomData,
                 })
    }
}

impl<T: FabricationEval> PuzzleVerifyView for FabricationVerifyView<T> {
    fn size(&self) -> RectSize<i32> { self.table.size() }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
            circuit_eval: Option<&CircuitEval>) {
        let (time_step, values, errors) = if let Some(eval) = circuit_eval {
            let puzzle = eval.puzzle_eval::<T>();
            (Some(eval.time_step()), puzzle.table_values(), eval.errors())
        } else {
            (None, T::expected_table_values(), &[] as &[EvalError])
        };
        self.table.draw(resources, matrix, time_step, values, errors);
    }
}

//===========================================================================//

const TABLE_COLUMN_WIDTH: i32 = 60;
const TABLE_ROW_HEIGHT: i32 = 20;
const TABLE_FONT_SIZE: f32 = 16.0;

pub struct FabricationTable {
    rect: Rect<i32>,
    column_names: &'static [&'static str],
    num_rows: usize,
}

impl FabricationTable {
    pub fn new(right_bottom: Point2<i32>,
               column_names: &'static [&'static str],
               expected_values: &[u64])
               -> FabricationTable {
        let num_cols = column_names.len();
        assert_eq!(expected_values.len() % num_cols, 0);
        let num_rows = expected_values.len() / num_cols;
        let height = TABLE_ROW_HEIGHT * ((num_rows as i32) + 1);
        let width = TABLE_COLUMN_WIDTH * (num_cols as i32);
        let rect = Rect::new(right_bottom.x - width,
                             right_bottom.y - height,
                             width,
                             height);
        FabricationTable {
            rect,
            column_names,
            num_rows,
        }
    }

    pub fn size(&self) -> RectSize<i32> { self.rect.size() }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                time_step: Option<u32>, values: &[u64], errors: &[EvalError]) {
        let rect = self.rect.as_f32();
        let column_width = rect.width / (self.column_names.len() as f32);
        let row_height = TABLE_ROW_HEIGHT as f32;
        for (index, column_name) in self.column_names.iter().enumerate() {
            let font = resources.fonts().roman();
            font.draw(matrix,
                      TABLE_FONT_SIZE,
                      Align::MidCenter,
                      (rect.x + ((index as f32) + 0.5) * column_width,
                       rect.y + 0.5 * row_height),
                      &column_name);
        }
        let num_columns = self.column_names.len();

        let mut error_rows = HashSet::<usize>::new();
        for error in errors.iter() {
            error_rows.insert(error.time_step as usize);
        }

        for row in 0..self.num_rows {
            let color = if Some(row as u32) == time_step {
                Color3::new(0.7, 0.7, 1.0)
            } else if error_rows.contains(&row) {
                Color3::new(0.7, 0.3, 0.3)
            } else {
                Color3::new(0.1, 0.1, 0.1)
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
