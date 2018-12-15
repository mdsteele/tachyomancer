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
use cgmath::{Deg, Matrix4, vec3};
use tachy::font::Align;
use tachy::geom::Rect;
use tachy::gui::{Event, Resources};
use tachy::save::Puzzle;
use tachy::state::GameState;

//===========================================================================//

const CIRCUIT_LIST_WIDTH: i32 = 200;
const ELEMENT_SPACING: i32 = 18;
const PUZZLE_LIST_WIDTH: i32 = 250;

const GRAPH_LABEL_FONT_SIZE: f32 = 14.0;
const GRAPH_INNER_MARGIN: i32 = 10;
const GRAPH_LABEL_MARGIN: i32 = 18;
const GRAPH_TICK_STEP: i32 = 10;
const GRAPH_TICK_LENGTH: f32 = 4.0;
const GRAPH_TICK_THICKNESS: f32 = 2.0;

//===========================================================================//

#[derive(Clone, Copy)]
pub enum PuzzlesAction {
    Edit,
    New,
}

//===========================================================================//

pub struct PuzzlesView {
    puzzle_list: ListView<Puzzle>,
    circuit_list: ListView<String>,
    graph: GraphView,
    edit_button: Button,
    new_button: Button,
}

impl PuzzlesView {
    pub fn new(rect: Rect<i32>, state: &GameState) -> PuzzlesView {
        // TODO: Filter puzzles based on what's unlocked
        let puzzle_list_items = Puzzle::all()
            .map(|puzzle| {
                     let mut label = puzzle.title().to_string();
                     if !state.is_puzzle_solved(puzzle) {
                         label = format!("* {}", label);
                     }
                     (puzzle, label)
                 })
            .collect();
        let semi_height = (rect.height - ELEMENT_SPACING) / 2;
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
                                                      semi_height,
                                                  CIRCUIT_LIST_WIDTH,
                                                  semi_height),
                                        state.circuit_name(),
                                        circuit_list_items(state)),
            graph: GraphView::new(Rect::new(rect.right() - semi_height,
                                            rect.y,
                                            semi_height,
                                            semi_height)),
            edit_button: Button::new(Rect::new(rect.right() - 80,
                                               rect.bottom() - 40,
                                               80,
                                               40),
                                     "Edit",
                                     PuzzlesAction::Edit),
            new_button: Button::new(Rect::new(rect.right() - 80,
                                              rect.bottom() - 80 -
                                                  ELEMENT_SPACING,
                                              80,
                                              40),
                                    "New",
                                    PuzzlesAction::New),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                state: &GameState) {
        let puzzle = state.current_puzzle();
        self.puzzle_list.draw(resources, matrix, &puzzle);
        let graph_points = state.puzzle_graph_points(puzzle);
        self.graph.draw(resources, matrix, puzzle, graph_points);
        self.circuit_list.draw(resources, matrix, state.circuit_name());
        self.edit_button.draw(resources, matrix);
        self.new_button.draw(resources, matrix);
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
        if let Some(action) = self.new_button.handle_event(event) {
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

struct Button {
    rect: Rect<i32>,
    label: &'static str,
    action: PuzzlesAction,
}

impl Button {
    pub fn new(rect: Rect<i32>, label: &'static str, action: PuzzlesAction)
               -> Button {
        Button {
            rect,
            label,
            action,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let color = (0.7, 0.1, 0.1);
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        resources.fonts().roman().draw(&matrix,
                                       20.0,
                                       Align::MidCenter,
                                       (rect.x + 0.5 * rect.width,
                                        rect.y + 0.5 * rect.height),
                                       self.label);
    }

    pub fn handle_event(&mut self, event: &Event) -> Option<PuzzlesAction> {
        match event {
            Event::MouseDown(mouse) => {
                if mouse.left && self.rect.contains_point(mouse.pt) {
                    return Some(self.action);
                }
            }
            _ => {}
        }
        return None;
    }
}

//===========================================================================//

struct GraphView {
    rect: Rect<i32>,
}

impl GraphView {
    pub fn new(rect: Rect<i32>) -> GraphView { GraphView { rect } }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                puzzle: Puzzle, points: &[(i32, i32)]) {
        let color = (0.1, 0.7, 0.4);
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);

        // If the puzzle hasn't been solved yet, don't draw a graph.
        let font = resources.fonts().roman();
        if points.is_empty() {
            font.draw(matrix,
                      20.0,
                      Align::MidCenter,
                      (rect.x + 0.5 * rect.width,
                       rect.y + 0.5 * rect.height - 12.0),
                      "COMPLETE THIS TASK TO");
            font.draw(matrix,
                      20.0,
                      Align::MidCenter,
                      (rect.x + 0.5 * rect.width,
                       rect.y + 0.5 * rect.height + 12.0),
                      "VIEW OPTIMIZATION GRAPH");
            return;
        }

        // Draw the graph data:
        let graph_rect = Rect::new(self.rect.x + GRAPH_INNER_MARGIN,
                                   self.rect.y + GRAPH_INNER_MARGIN,
                                   self.rect.width - 2 * GRAPH_INNER_MARGIN -
                                       GRAPH_LABEL_MARGIN,
                                   self.rect.height - 2 * GRAPH_INNER_MARGIN -
                                       GRAPH_LABEL_MARGIN);
        let graph_rect = graph_rect.as_f32();
        let color = (0.1, 0.1, 0.1);
        resources.shaders().solid().fill_rect(&matrix, color, graph_rect);
        let graph_bounds = puzzle.graph_bounds();
        let color = (0.9, 0.1, 0.1);
        for &(pt_x, pt_y) in points.iter() {
            let rel_x = graph_rect.width *
                ((pt_x as f32) / (graph_bounds.0 as f32));
            let rel_y = graph_rect.height *
                ((pt_y as f32) / (graph_bounds.1 as f32));
            let point_rect = Rect::new(graph_rect.x + rel_x,
                                       graph_rect.y,
                                       graph_rect.width - rel_x,
                                       graph_rect.height - rel_y);
            resources.shaders().solid().fill_rect(&matrix, color, point_rect);
        }

        // Draw axis ticks:
        let color = (0.1, 0.1, 0.1);
        let unit_span = (graph_rect.width - GRAPH_TICK_THICKNESS) /
            (graph_bounds.0 as f32);
        let mut tick = 0;
        while tick <= graph_bounds.0 {
            let tick_rect = Rect::new(graph_rect.x +
                                          (tick as f32) * unit_span,
                                      graph_rect.bottom(),
                                      GRAPH_TICK_THICKNESS,
                                      GRAPH_TICK_LENGTH);
            resources.shaders().solid().fill_rect(&matrix, color, tick_rect);
            tick += GRAPH_TICK_STEP;
        }
        let unit_span = (graph_rect.height - GRAPH_TICK_THICKNESS) /
            (graph_bounds.1 as f32);
        tick = 0;
        while tick <= graph_bounds.1 {
            let tick_rect = Rect::new(graph_rect.right(),
                                      graph_rect.bottom() -
                                          GRAPH_TICK_THICKNESS -
                                          (tick as f32) * unit_span,
                                      GRAPH_TICK_LENGTH,
                                      GRAPH_TICK_THICKNESS);
            resources.shaders().solid().fill_rect(&matrix, color, tick_rect);
            tick += GRAPH_TICK_STEP;
        }

        // Draw axis labels:
        font.draw(matrix,
                  GRAPH_LABEL_FONT_SIZE,
                  Align::BottomCenter,
                  (graph_rect.x + 0.5 * graph_rect.width,
                   graph_rect.bottom() + GRAPH_LABEL_MARGIN as f32),
                  "Size");
        let side_matrix = matrix *
            Matrix4::from_translation(vec3(graph_rect.right(),
                                           graph_rect.bottom(),
                                           0.0)) *
            Matrix4::from_angle_z(Deg(-90.0));
        font.draw(&side_matrix,
                  GRAPH_LABEL_FONT_SIZE,
                  Align::BottomCenter,
                  (0.5 * graph_rect.height, GRAPH_LABEL_MARGIN as f32),
                  puzzle.score_units());
    }
}

//===========================================================================//
