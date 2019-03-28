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
use super::super::button::TextButton;
use super::super::paragraph::Paragraph;
use cgmath::{Deg, Matrix4};
use std::cell::RefCell;
use tachy::font::Align;
use tachy::geom::{AsFloat, Color4, MatrixExt, Rect};
use tachy::gui::{Event, Resources, Ui};
use tachy::save::{Prefs, Puzzle};
use tachy::state::GameState;

//===========================================================================//

const CIRCUIT_LIST_WIDTH: i32 = 240;
const ELEMENT_SPACING: i32 = 18;
const PUZZLE_LIST_WIDTH: i32 = 250;

const DESCRIPTION_FONT_SIZE: f32 = 20.0;
const DESCRIPTION_INNER_MARGIN_HORZ: i32 = 14;
const DESCRIPTION_INNER_MARGIN_VERT: i32 = 10;
const DESCRIPTION_LINE_HEIGHT: f32 = 22.0;

const GRAPH_LABEL_FONT_SIZE: f32 = 14.0;
const GRAPH_INNER_MARGIN: i32 = 10;
const GRAPH_LABEL_MARGIN: i32 = 18;
const GRAPH_TICK_STEP: i32 = 10;
const GRAPH_TICK_LENGTH: f32 = 4.0;
const GRAPH_TICK_THICKNESS: f32 = 2.0;

//===========================================================================//

#[derive(Clone, Copy)]
pub enum PuzzlesAction {
    Copy,
    Delete,
    Edit,
    New,
    Rename,
}

//===========================================================================//

pub struct PuzzlesView {
    puzzle_list: ListView<Puzzle>,
    circuit_list: ListView<String>,
    description: DescriptionView,
    graph: GraphView,
    copy_button: TextButton<PuzzlesAction>,
    delete_button: TextButton<PuzzlesAction>,
    edit_button: TextButton<PuzzlesAction>,
    new_button: TextButton<PuzzlesAction>,
    rename_button: TextButton<PuzzlesAction>,
}

impl PuzzlesView {
    pub fn new(rect: Rect<i32>, state: &GameState) -> PuzzlesView {
        let semi_height = (rect.height - ELEMENT_SPACING) / 2;
        PuzzlesView {
            puzzle_list: ListView::new(Rect::new(rect.x,
                                                 rect.y,
                                                 PUZZLE_LIST_WIDTH,
                                                 rect.height),
                                       &state.current_puzzle(),
                                       puzzle_list_items(state)),
            circuit_list: ListView::new(Rect::new(rect.x + PUZZLE_LIST_WIDTH +
                                                      ELEMENT_SPACING,
                                                  rect.bottom() -
                                                      semi_height,
                                                  CIRCUIT_LIST_WIDTH,
                                                  semi_height),
                                        state.circuit_name(),
                                        circuit_list_items(state)),
            description:
                DescriptionView::new(Rect::new(rect.x + PUZZLE_LIST_WIDTH +
                                                   ELEMENT_SPACING,
                                               rect.y,
                                               rect.width - PUZZLE_LIST_WIDTH -
                                                   semi_height -
                                                   2 * ELEMENT_SPACING,
                                               semi_height)),
            graph: GraphView::new(Rect::new(rect.right() - semi_height,
                                            rect.y,
                                            semi_height,
                                            semi_height)),
            edit_button: TextButton::new(Rect::new(rect.right() - 80,
                                                   rect.bottom() - 40,
                                                   80,
                                                   40),
                                         "Edit",
                                         PuzzlesAction::Edit),
            new_button: TextButton::new(Rect::new(rect.right() - 80,
                                                  rect.bottom() - 80 -
                                                      ELEMENT_SPACING,
                                                  80,
                                                  40),
                                        "New",
                                        PuzzlesAction::New),
            delete_button: TextButton::new(Rect::new(rect.right() - 80,
                                                     rect.bottom() - 120 -
                                                         2 * ELEMENT_SPACING,
                                                     80,
                                                     40),
                                           "Delete",
                                           PuzzlesAction::Delete),
            rename_button: TextButton::new(Rect::new(rect.right() - 80,
                                                     rect.bottom() - 160 -
                                                         3 * ELEMENT_SPACING,
                                                     80,
                                                     40),
                                           "Rename",
                                           PuzzlesAction::Rename),
            copy_button: TextButton::new(Rect::new(rect.right() - 80,
                                                   rect.bottom() - 200 -
                                                       4 * ELEMENT_SPACING,
                                                   80,
                                                   40),
                                         "Copy",
                                         PuzzlesAction::Copy),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                state: &GameState) {
        let puzzle = state.current_puzzle();
        self.puzzle_list.draw(resources, matrix, &puzzle);
        self.description.draw(resources, matrix, puzzle, state.prefs());
        let scores = state.puzzle_scores(puzzle);
        self.graph.draw(resources, matrix, puzzle, scores);
        self.circuit_list.draw(resources, matrix, state.circuit_name());
        // TODO: edit/delete/rename/copy buttons are not always enabled
        self.edit_button.draw(resources, matrix, true);
        self.new_button.draw(resources, matrix, true);
        self.delete_button.draw(resources, matrix, true);
        self.rename_button.draw(resources, matrix, true);
        self.copy_button.draw(resources, matrix, true);
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui,
                    state: &mut GameState)
                    -> Option<PuzzlesAction> {
        if let Some(puzzle) = self.puzzle_list
            .on_event(event, &state.current_puzzle())
        {
            state.set_current_puzzle(puzzle);
            self.update_circuit_list(state);
        }
        if let Some(circuit_name) =
            self.circuit_list.on_event(event, state.circuit_name())
        {
            state.set_circuit_name(circuit_name);
        }
        // TODO: edit/delete/rename/copy buttons are not always enabled
        if let Some(action) = self.edit_button.on_event(event, ui, true) {
            return Some(action);
        }
        if let Some(action) = self.new_button.on_event(event, ui, true) {
            return Some(action);
        }
        if let Some(action) = self.delete_button.on_event(event, ui, true) {
            return Some(action);
        }
        if let Some(action) = self.rename_button.on_event(event, ui, true) {
            return Some(action);
        }
        if let Some(action) = self.copy_button.on_event(event, ui, true) {
            return Some(action);
        }
        return None;
    }

    pub fn update_circuit_list(&mut self, state: &GameState) {
        self.circuit_list
            .set_items(state.circuit_name(), circuit_list_items(state));
    }

    pub fn update_puzzle_list(&mut self, state: &GameState) {
        self.puzzle_list
            .set_items(&state.current_puzzle(), puzzle_list_items(state));
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

fn puzzle_list_items(state: &GameState) -> Vec<(Puzzle, String)> {
    Puzzle::all()
        .filter(|&puzzle| state.is_puzzle_unlocked(puzzle))
        .map(|puzzle| {
                 let mut label = puzzle.title().to_string();
                 if !state.is_puzzle_solved(puzzle) {
                     label = format!("* {}", label);
                 }
                 (puzzle, label)
             })
        .collect()
}

//===========================================================================//

struct DescriptionView {
    rect: Rect<i32>,
    cache: RefCell<Option<(Puzzle, Paragraph)>>,
}

impl DescriptionView {
    pub fn new(rect: Rect<i32>) -> DescriptionView {
        DescriptionView {
            rect,
            cache: RefCell::new(None),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                puzzle: Puzzle, prefs: &Prefs) {
        resources.shaders().ui().draw_box2(matrix,
                                           &self.rect.as_f32(),
                                           &Color4::ORANGE2,
                                           &Color4::CYAN2,
                                           &Color4::PURPLE0.with_alpha(0.8));

        let mut cached = self.cache.borrow_mut();
        match cached.as_ref() {
            Some(&(puzz, ref paragraph)) if puzz == puzzle => {
                self.draw_text(resources, matrix, paragraph);
                return;
            }
            _ => {}
        }

        debug_log!("Recompiling description paragraph");
        let width = (self.rect.width - 2 * DESCRIPTION_INNER_MARGIN_HORZ) as
            f32;
        let paragraph = Paragraph::compile(DESCRIPTION_FONT_SIZE,
                                           DESCRIPTION_LINE_HEIGHT,
                                           width,
                                           prefs,
                                           puzzle.description());
        self.draw_text(resources, matrix, &paragraph);
        *cached = Some((puzzle, paragraph));
    }

    fn draw_text(&self, resources: &Resources, matrix: &Matrix4<f32>,
                 paragraph: &Paragraph) {
        let left = (self.rect.x + DESCRIPTION_INNER_MARGIN_HORZ) as f32;
        let top = (self.rect.y + DESCRIPTION_INNER_MARGIN_VERT) as f32;
        paragraph.draw(resources, matrix, (left, top));
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
            Matrix4::trans2(graph_rect.right(), graph_rect.bottom()) *
            Matrix4::from_angle_z(Deg(-90.0));
        font.draw(&side_matrix,
                  GRAPH_LABEL_FONT_SIZE,
                  Align::BottomCenter,
                  (0.5 * graph_rect.height, GRAPH_LABEL_MARGIN as f32),
                  puzzle.score_units());
    }
}

//===========================================================================//
