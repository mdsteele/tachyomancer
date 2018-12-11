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

use super::list::ListView;
use cgmath::Matrix4;
use tachy::font::Align;
use tachy::gui::{Event, Resources};
use tachy::save::Puzzle;
use tachy::state::{GameState, Rect};

//===========================================================================//

const CIRCUIT_LIST_WIDTH: i32 = 200;
const ELEMENT_SPACING: i32 = 18;
const PUZZLE_LIST_WIDTH: i32 = 250;

//===========================================================================//

pub enum PuzzlesAction {
    Edit,
}

//===========================================================================//

pub struct PuzzlesView {
    puzzle_list: ListView<Puzzle>,
    circuit_list: ListView<String>,
    edit_button: EditButton,
}

impl PuzzlesView {
    pub fn new(rect: Rect<i32>, state: &GameState) -> PuzzlesView {
        // TODO: Filter puzzles based on what's unlocked
        let puzzle_list_items = Puzzle::all()
            .map(|puzzle| (puzzle, puzzle.title().to_string()))
            .collect();
        let circuit_list_height = (rect.height - ELEMENT_SPACING) / 2;
        PuzzlesView {
            puzzle_list: ListView::new(Rect::new(rect.x,
                                                 rect.y,
                                                 PUZZLE_LIST_WIDTH,
                                                 rect.height),
                                       &state.current_puzzle(),
                                       puzzle_list_items),
            circuit_list: ListView::new(Rect::new(rect.x + PUZZLE_LIST_WIDTH +
                                                      ELEMENT_SPACING,
                                                  rect.bottom() -
                                                      circuit_list_height,
                                                  CIRCUIT_LIST_WIDTH,
                                                  circuit_list_height),
                                        state.circuit_name(),
                                        circuit_list_items(state)),
            edit_button: EditButton::new(Rect::new(rect.right() - 80,
                                                   rect.bottom() - 40,
                                                   80,
                                                   40)),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                state: &GameState) {
        self.puzzle_list.draw(resources, matrix, &state.current_puzzle());
        self.circuit_list.draw(resources, matrix, state.circuit_name());
        self.edit_button.draw(resources, matrix);
    }

    pub fn handle_event(&mut self, event: &Event, state: &mut GameState)
                        -> Option<PuzzlesAction> {
        if let Some(puzzle) =
            self.puzzle_list.handle_event(event, &state.current_puzzle())
        {
            state.set_current_puzzle(puzzle);
            self.circuit_list
                .set_items(state.circuit_name(), circuit_list_items(state));
        }
        if let Some(circuit_name) =
            self.circuit_list.handle_event(event, state.circuit_name())
        {
            state.set_circuit_name(circuit_name);
        }
        if let Some(action) = self.edit_button.handle_event(event) {
            return Some(action);
        }
        return None;
    }

    pub fn unfocus(&mut self) {
        self.puzzle_list.unfocus();
        self.circuit_list.unfocus();
    }
}

fn circuit_list_items(state: &GameState) -> Vec<(String, String)> {
    if let Some(profile) = state.profile() {
        profile
            .circuit_names(profile.current_puzzle())
            .map(|name| (name.to_string(), name.to_string()))
            .collect()
    } else {
        Vec::new()
    }
}

//===========================================================================//

struct EditButton {
    rect: Rect<i32>,
}

impl EditButton {
    pub fn new(rect: Rect<i32>) -> EditButton { EditButton { rect } }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let color = (0.7, 0.1, 0.1);
        let rect = (self.rect.x as f32,
                    self.rect.y as f32,
                    self.rect.width as f32,
                    self.rect.height as f32);
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        resources.fonts().roman().draw(&matrix,
                                       20.0,
                                       Align::Center,
                                       ((self.rect.x as f32) +
                                            0.5 * (self.rect.width as f32),
                                        (self.rect.y as f32) +
                                            0.5 * (self.rect.height as f32) -
                                            10.0),
                                       "Edit");
    }

    pub fn handle_event(&mut self, event: &Event) -> Option<PuzzlesAction> {
        match event {
            Event::MouseDown(mouse) => {
                if mouse.left && self.rect.contains_point(mouse.pt) {
                    return Some(PuzzlesAction::Edit);
                }
            }
            _ => {}
        }
        return None;
    }
}

//===========================================================================//
