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

use super::shared::{FabricationTable, PuzzleVerifyView};
use cgmath::{Matrix4, Point2};
use tachy::geom::RectSize;
use tachy::gui::Resources;
use tachy::state::{CircuitEval, EvalError, TutorialAddEval, TutorialMuxEval,
                   TutorialOrEval, TutorialXorEval};

//===========================================================================//

pub struct TutorialOrVerifyView {
    table: FabricationTable,
}

impl TutorialOrVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<PuzzleVerifyView> {
        let table =
            FabricationTable::new(right_bottom,
                                  TutorialOrEval::TABLE_COLUMN_NAMES,
                                  TutorialOrEval::EXPECTED_TABLE_VALUES);
        Box::new(TutorialOrVerifyView { table })
    }
}

impl PuzzleVerifyView for TutorialOrVerifyView {
    fn size(&self) -> RectSize<i32> { self.table.size() }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
            circuit_eval: Option<&CircuitEval>) {
        let (time_step, values, errors) = if let Some(eval) = circuit_eval {
            let puzzle = eval.puzzle_eval::<TutorialOrEval>();
            (Some(eval.time_step()), puzzle.table_values(), eval.errors())
        } else {
            (None, TutorialOrEval::EXPECTED_TABLE_VALUES, &[] as &[EvalError])
        };
        self.table.draw(resources, matrix, time_step, values, errors);
    }
}

//===========================================================================//

pub struct TutorialXorVerifyView {
    table: FabricationTable,
}

impl TutorialXorVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<PuzzleVerifyView> {
        let table =
            FabricationTable::new(right_bottom,
                                  TutorialXorEval::TABLE_COLUMN_NAMES,
                                  TutorialXorEval::EXPECTED_TABLE_VALUES);
        Box::new(TutorialXorVerifyView { table })
    }
}

impl PuzzleVerifyView for TutorialXorVerifyView {
    fn size(&self) -> RectSize<i32> { self.table.size() }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
            circuit_eval: Option<&CircuitEval>) {
        let (time_step, values, errors) = if let Some(eval) = circuit_eval {
            let puzzle = eval.puzzle_eval::<TutorialXorEval>();
            (Some(eval.time_step()), puzzle.table_values(), eval.errors())
        } else {
            (None, TutorialXorEval::EXPECTED_TABLE_VALUES, &[] as &[EvalError])
        };
        self.table.draw(resources, matrix, time_step, values, errors);
    }
}

//===========================================================================//

pub struct TutorialMuxVerifyView {
    table: FabricationTable,
}

impl TutorialMuxVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<PuzzleVerifyView> {
        let table =
            FabricationTable::new(right_bottom,
                                  TutorialMuxEval::TABLE_COLUMN_NAMES,
                                  TutorialMuxEval::EXPECTED_TABLE_VALUES);
        Box::new(TutorialMuxVerifyView { table })
    }
}

impl PuzzleVerifyView for TutorialMuxVerifyView {
    fn size(&self) -> RectSize<i32> { self.table.size() }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
            circuit_eval: Option<&CircuitEval>) {
        let (time_step, values, errors) = if let Some(eval) = circuit_eval {
            let puzzle = eval.puzzle_eval::<TutorialMuxEval>();
            (Some(eval.time_step()), puzzle.table_values(), eval.errors())
        } else {
            (None, TutorialMuxEval::EXPECTED_TABLE_VALUES, &[] as &[EvalError])
        };
        self.table.draw(resources, matrix, time_step, values, errors);
    }
}

//===========================================================================//

pub struct TutorialAddVerifyView {
    table: FabricationTable,
}

impl TutorialAddVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<PuzzleVerifyView> {
        let table =
            FabricationTable::new(right_bottom,
                                  TutorialAddEval::TABLE_COLUMN_NAMES,
                                  TutorialAddEval::EXPECTED_TABLE_VALUES);
        Box::new(TutorialAddVerifyView { table })
    }
}

impl PuzzleVerifyView for TutorialAddVerifyView {
    fn size(&self) -> RectSize<i32> { self.table.size() }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
            circuit_eval: Option<&CircuitEval>) {
        let (time_step, values, errors) = if let Some(eval) = circuit_eval {
            let puzzle = eval.puzzle_eval::<TutorialAddEval>();
            (Some(eval.time_step()), puzzle.table_values(), eval.errors())
        } else {
            (None, TutorialAddEval::EXPECTED_TABLE_VALUES, &[] as &[EvalError])
        };
        self.table.draw(resources, matrix, time_step, values, errors);
    }
}

//===========================================================================//
