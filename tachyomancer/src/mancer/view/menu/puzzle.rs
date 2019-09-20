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

use super::super::button::TextButton;
use super::super::chip::ChipModel;
use super::super::paragraph::Paragraph;
use super::super::wire::WireModel;
use super::list::{ListIcon, ListView};
use crate::mancer::font::Align;
use crate::mancer::gl::FrameBuffer;
use crate::mancer::gui::{Event, Resources, Ui};
use crate::mancer::save::Prefs;
use crate::mancer::state::GameState;
use cgmath::{vec2, Deg, Matrix4};
use std::cell::RefCell;
use tachy::geom::{AsFloat, Color3, Color4, MatrixExt, Rect, RectSize};
use tachy::save::{Conversation, Puzzle, PuzzleKind};
use tachy::state::{EditGrid, PuzzleExt, WireColor};

//===========================================================================//

const BUTTON_WIDTH: i32 = 90;
const CIRCUIT_LIST_WIDTH: i32 = 240;
const ELEMENT_SPACING: i32 = 18;
const PUZZLE_LIST_WIDTH: i32 = 250;

const DESCRIPTION_FONT_SIZE: f32 = 20.0;
const DESCRIPTION_INNER_MARGIN_HORZ: f32 = 14.0;
const DESCRIPTION_INNER_MARGIN_VERT: f32 = 10.0;
const DESCRIPTION_LINE_HEIGHT: f32 = 22.0;

const GRAPH_LABEL_FONT_SIZE: f32 = 14.0;
const GRAPH_INNER_MARGIN: i32 = 10;
const GRAPH_LABEL_MARGIN: i32 = 18;
const GRAPH_TICK_STEP: i32 = 10;
const GRAPH_TICK_LENGTH: f32 = 4.0;
const GRAPH_TICK_THICKNESS: f32 = 2.0;

const PREVIEW_INNER_MARGIN: f32 = 14.0;
const PREVIEW_MAX_GRID_CELL_SIZE: f32 = 48.0;

//===========================================================================//

#[derive(Clone, Copy)]
pub enum PuzzlesAction {
    GoToConversation(Conversation),
    Copy,
    Delete,
    Edit,
    Rename,
}

//===========================================================================//

pub struct PuzzlesView {
    puzzle_list: ListView<Puzzle>,
    circuit_list: ListView<String>,
    back_button: TextButton<()>,
    description: DescriptionView,
    graph: GraphView,
    preview: CircuitPreviewView,
    edit_button: TextButton<PuzzlesAction>,
    rename_button: TextButton<PuzzlesAction>,
    copy_button: TextButton<PuzzlesAction>,
    delete_button: TextButton<PuzzlesAction>,
}

impl PuzzlesView {
    pub fn new(
        window_size: RectSize<i32>,
        rect: Rect<i32>,
        ui: &mut Ui,
        state: &GameState,
    ) -> PuzzlesView {
        let semi_height = (rect.height - ELEMENT_SPACING) / 2;
        let button_height = (semi_height - 3 * ELEMENT_SPACING) / 4;
        let buttons_left = rect.right() - BUTTON_WIDTH;
        let buttons_top = rect.y + semi_height + ELEMENT_SPACING;

        let puzzle_list_rect =
            Rect::new(rect.x, rect.y, PUZZLE_LIST_WIDTH, rect.height);
        let graph_rect = Rect::new(
            rect.right() - semi_height,
            rect.y,
            semi_height,
            semi_height,
        );
        let description_rect = Rect::new(
            puzzle_list_rect.right() + ELEMENT_SPACING,
            rect.y,
            graph_rect.x - puzzle_list_rect.right() - 2 * ELEMENT_SPACING,
            semi_height,
        );
        let back_button_rect = Rect::new(
            description_rect.x,
            description_rect.bottom() - 40,
            BUTTON_WIDTH,
            40,
        );
        let circuit_list_rect = Rect::new(
            rect.x + PUZZLE_LIST_WIDTH + ELEMENT_SPACING,
            rect.bottom() - semi_height,
            CIRCUIT_LIST_WIDTH,
            semi_height,
        );
        let preview_rect = Rect::new(
            circuit_list_rect.right() + ELEMENT_SPACING,
            circuit_list_rect.y,
            buttons_left - circuit_list_rect.right() - 2 * ELEMENT_SPACING,
            semi_height,
        );

        let mut button_rect =
            Rect::new(buttons_left, buttons_top, BUTTON_WIDTH, button_height);
        let edit_button =
            TextButton::new(button_rect, "Edit", PuzzlesAction::Edit);
        button_rect.y += button_height + ELEMENT_SPACING;
        let rename_button =
            TextButton::new(button_rect, "Rename", PuzzlesAction::Rename);
        button_rect.y += button_height + ELEMENT_SPACING;
        let copy_button =
            TextButton::new(button_rect, "Copy", PuzzlesAction::Copy);
        button_rect.y += button_height + ELEMENT_SPACING;
        let delete_button =
            TextButton::new(button_rect, "Delete", PuzzlesAction::Delete);

        PuzzlesView {
            puzzle_list: ListView::new(
                puzzle_list_rect,
                ui,
                puzzle_list_items(state),
                &state.current_puzzle(),
            ),
            circuit_list: ListView::new(
                circuit_list_rect,
                ui,
                circuit_list_items(state),
                state.circuit_name(),
            ),
            description: DescriptionView::new(description_rect),
            graph: GraphView::new(graph_rect),
            preview: CircuitPreviewView::new(window_size, preview_rect),
            edit_button,
            rename_button,
            copy_button,
            delete_button,
            back_button: TextButton::new(back_button_rect, "Back", ()),
        }
    }

    fn copy_and_delete_enabled(&self, state: &GameState) -> bool {
        !state.circuit_name().is_empty()
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        state: &GameState,
    ) {
        let puzzle = state.current_puzzle();
        self.puzzle_list.draw(resources, matrix, &puzzle);
        self.description.draw(resources, matrix, puzzle, state.prefs());
        self.back_button.draw(resources, matrix, true);
        let scores = state.puzzle_scores(puzzle);
        self.graph.draw(resources, matrix, puzzle, scores);
        self.circuit_list.draw(resources, matrix, state.circuit_name());
        self.preview.draw(resources, matrix, state);
        self.edit_button.draw(resources, matrix, true);
        self.rename_button.draw(resources, matrix, true);
        let enabled = self.copy_and_delete_enabled(state);
        self.copy_button.draw(resources, matrix, enabled);
        self.delete_button.draw(resources, matrix, enabled);
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        state: &mut GameState,
    ) -> Option<PuzzlesAction> {
        if let Some(puzzle) =
            self.puzzle_list.on_event(event, ui, &state.current_puzzle())
        {
            state.set_current_puzzle(puzzle);
            ui.request_redraw();
            self.update_circuit_list(ui, state);
        }
        if let Some(circuit_name) =
            self.circuit_list.on_event(event, ui, state.circuit_name())
        {
            state.set_circuit_name(circuit_name);
            ui.request_redraw();
        }
        if let Some(()) = self.back_button.on_event(event, ui, true) {
            let puzzle = state.current_puzzle();
            for &conv in puzzle.origin_conversations() {
                if state.is_conversation_unlocked(conv) {
                    return Some(PuzzlesAction::GoToConversation(conv));
                }
            }
        }
        if let Some(action) = self.edit_button.on_event(event, ui, true) {
            return Some(action);
        }
        if let Some(action) = self.rename_button.on_event(event, ui, true) {
            return Some(action);
        }
        let enabled = self.copy_and_delete_enabled(state);
        if let Some(action) = self.copy_button.on_event(event, ui, enabled) {
            return Some(action);
        }
        if let Some(action) = self.delete_button.on_event(event, ui, enabled) {
            return Some(action);
        }
        return None;
    }

    pub fn update_circuit_list(&mut self, ui: &mut Ui, state: &GameState) {
        self.circuit_list.set_items(
            ui,
            circuit_list_items(state),
            state.circuit_name(),
        );
    }

    pub fn update_puzzle_list(&mut self, ui: &mut Ui, state: &GameState) {
        self.puzzle_list.set_items(
            ui,
            puzzle_list_items(state),
            &state.current_puzzle(),
        );
    }
}

fn circuit_list_items(
    state: &GameState,
) -> Vec<(String, String, bool, Option<ListIcon>)> {
    let mut items =
        vec![("".to_string(), "    [New Circuit]".to_string(), false, None)];
    if let Some(profile) = state.profile() {
        items.extend(
            profile
                .circuit_names(profile.current_puzzle())
                .map(|name| (name.to_string(), name.to_string(), false, None)),
        )
    }
    items
}

fn puzzle_list_items(
    state: &GameState,
) -> Vec<(Puzzle, String, bool, Option<ListIcon>)> {
    Puzzle::all()
        .filter(|&puzzle| state.is_puzzle_unlocked(puzzle))
        .map(|puzzle| {
            let label = puzzle.title().to_string();
            let icon = match puzzle.kind() {
                PuzzleKind::Automate => ListIcon::Automate,
                PuzzleKind::Command => ListIcon::Command,
                PuzzleKind::Fabricate => ListIcon::Fabricate,
                PuzzleKind::Sandbox => ListIcon::Sandbox,
                PuzzleKind::Tutorial => ListIcon::Tutorial,
            };
            (puzzle, label, !state.is_puzzle_solved(puzzle), Some(icon))
        })
        .collect()
}

//===========================================================================//

struct DescriptionView {
    rect: Rect<f32>,
    cache: RefCell<Option<(Puzzle, Paragraph)>>,
}

impl DescriptionView {
    pub fn new(rect: Rect<i32>) -> DescriptionView {
        DescriptionView { rect: rect.as_f32(), cache: RefCell::new(None) }
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        puzzle: Puzzle,
        prefs: &Prefs,
    ) {
        resources.shaders().ui().draw_box2(
            matrix,
            &self.rect,
            &Color4::ORANGE2,
            &Color4::CYAN2,
            &Color4::PURPLE0_TRANSLUCENT,
        );

        let mut cached = self.cache.borrow_mut();
        match cached.as_ref() {
            Some(&(puzz, ref paragraph)) if puzz == puzzle => {
                self.draw_text(resources, matrix, paragraph);
                return;
            }
            _ => {}
        }

        debug_log!("Recompiling description paragraph");
        let width = self.rect.width - 2.0 * DESCRIPTION_INNER_MARGIN_HORZ;
        let paragraph = Paragraph::compile(
            DESCRIPTION_FONT_SIZE,
            DESCRIPTION_LINE_HEIGHT,
            width,
            prefs,
            puzzle.description(),
        );
        self.draw_text(resources, matrix, &paragraph);
        *cached = Some((puzzle, paragraph));
    }

    fn draw_text(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        paragraph: &Paragraph,
    ) {
        let left = self.rect.x + DESCRIPTION_INNER_MARGIN_HORZ;
        let top = self.rect.y + DESCRIPTION_INNER_MARGIN_VERT;
        paragraph.draw(resources, matrix, (left, top));
    }
}

//===========================================================================//

struct GraphView {
    rect: Rect<i32>,
}

impl GraphView {
    pub fn new(rect: Rect<i32>) -> GraphView {
        GraphView { rect }
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        puzzle: Puzzle,
        points: &[(i32, i32)],
    ) {
        let color = Color3::new(0.1, 0.7, 0.4);
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);

        // If the puzzle hasn't been solved yet, don't draw a graph.
        let font = resources.fonts().roman();
        if points.is_empty() {
            font.draw(
                matrix,
                20.0,
                Align::MidCenter,
                (rect.x + 0.5 * rect.width, rect.y + 0.5 * rect.height - 12.0),
                "COMPLETE THIS TASK TO",
            );
            font.draw(
                matrix,
                20.0,
                Align::MidCenter,
                (rect.x + 0.5 * rect.width, rect.y + 0.5 * rect.height + 12.0),
                "VIEW OPTIMIZATION GRAPH",
            );
            return;
        }

        // Draw the graph data:
        let graph_rect = Rect::new(
            self.rect.x + GRAPH_INNER_MARGIN,
            self.rect.y + GRAPH_INNER_MARGIN,
            self.rect.width - 2 * GRAPH_INNER_MARGIN - GRAPH_LABEL_MARGIN,
            self.rect.height - 2 * GRAPH_INNER_MARGIN - GRAPH_LABEL_MARGIN,
        );
        let graph_rect = graph_rect.as_f32();
        let color = Color3::new(0.1, 0.1, 0.1);
        resources.shaders().solid().fill_rect(&matrix, color, graph_rect);
        let graph_bounds = puzzle.graph_bounds();
        let color = Color3::new(0.9, 0.1, 0.1);
        for &(pt_x, pt_y) in points.iter() {
            let rel_x =
                graph_rect.width * ((pt_x as f32) / (graph_bounds.0 as f32));
            let rel_y =
                graph_rect.height * ((pt_y as f32) / (graph_bounds.1 as f32));
            let point_rect = Rect::new(
                graph_rect.x + rel_x,
                graph_rect.y,
                graph_rect.width - rel_x,
                graph_rect.height - rel_y,
            );
            resources.shaders().solid().fill_rect(&matrix, color, point_rect);
        }

        // Draw axis ticks:
        let color = Color3::new(0.1, 0.1, 0.1);
        let unit_span = (graph_rect.width - GRAPH_TICK_THICKNESS)
            / (graph_bounds.0 as f32);
        let mut tick = 0;
        while tick <= graph_bounds.0 {
            let tick_rect = Rect::new(
                graph_rect.x + (tick as f32) * unit_span,
                graph_rect.bottom(),
                GRAPH_TICK_THICKNESS,
                GRAPH_TICK_LENGTH,
            );
            resources.shaders().solid().fill_rect(&matrix, color, tick_rect);
            tick += GRAPH_TICK_STEP;
        }
        let unit_span = (graph_rect.height - GRAPH_TICK_THICKNESS)
            / (graph_bounds.1 as f32);
        tick = 0;
        while tick <= graph_bounds.1 {
            let tick_rect = Rect::new(
                graph_rect.right(),
                graph_rect.bottom()
                    - GRAPH_TICK_THICKNESS
                    - (tick as f32) * unit_span,
                GRAPH_TICK_LENGTH,
                GRAPH_TICK_THICKNESS,
            );
            resources.shaders().solid().fill_rect(&matrix, color, tick_rect);
            tick += GRAPH_TICK_STEP;
        }

        // Draw axis labels:
        font.draw(
            matrix,
            GRAPH_LABEL_FONT_SIZE,
            Align::BottomCenter,
            (
                graph_rect.x + 0.5 * graph_rect.width,
                graph_rect.bottom() + GRAPH_LABEL_MARGIN as f32,
            ),
            "Size",
        );
        let side_matrix = matrix
            * Matrix4::trans2(graph_rect.right(), graph_rect.bottom())
            * Matrix4::from_angle_z(Deg(-90.0));
        font.draw(
            &side_matrix,
            GRAPH_LABEL_FONT_SIZE,
            Align::BottomCenter,
            (0.5 * graph_rect.height, GRAPH_LABEL_MARGIN as f32),
            puzzle.score_units(),
        );
    }
}

//===========================================================================//

struct CircuitPreviewView {
    window_size: RectSize<i32>,
    rect: Rect<f32>,
    cache: RefCell<Option<(Puzzle, String, FrameBuffer)>>,
}

impl CircuitPreviewView {
    pub fn new(
        window_size: RectSize<i32>,
        rect: Rect<i32>,
    ) -> CircuitPreviewView {
        CircuitPreviewView {
            window_size,
            rect: rect.as_f32(),
            cache: RefCell::new(None),
        }
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        state: &GameState,
    ) {
        resources.shaders().ui().draw_box2(
            matrix,
            &self.rect,
            &Color4::ORANGE2,
            &Color4::CYAN2,
            &Color4::PURPLE0_TRANSLUCENT,
        );
        let puzzle = state.current_puzzle();
        let circuit_name = state.circuit_name();

        let mut cached = self.cache.borrow_mut();
        match cached.as_ref() {
            Some(&(puzz, ref name, ref fbo))
                if puzz == puzzle && name == circuit_name =>
            {
                self.draw_fbo(resources, matrix, fbo);
                return;
            }
            _ => {}
        }

        let fbo = self.generate_fbo(resources, state, puzzle, circuit_name);
        self.draw_fbo(resources, matrix, &fbo);
        *cached = Some((puzzle, circuit_name.to_string(), fbo));
    }

    fn draw_fbo(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        fbo: &FrameBuffer,
    ) {
        let left_top = self.rect.top_left()
            + vec2(PREVIEW_INNER_MARGIN, PREVIEW_INNER_MARGIN);
        let grayscale = false;
        resources.shaders().frame().draw(matrix, fbo, left_top, grayscale);
    }

    fn generate_fbo(
        &self,
        resources: &Resources,
        state: &GameState,
        puzzle: Puzzle,
        circuit_name: &str,
    ) -> FrameBuffer {
        debug_log!("Regenerating preview image");
        let fbo_size = self.rect.size().expand(-PREVIEW_INNER_MARGIN);
        let fbo = FrameBuffer::new(
            fbo_size.width as usize,
            fbo_size.height as usize,
        );
        fbo.bind();
        match state.load_edit_grid(puzzle, circuit_name) {
            Ok(grid) => self.draw_edit_grid(resources, fbo_size, &grid),
            Err(error) => {
                self.draw_error_paragraph(
                    resources,
                    fbo_size,
                    &error,
                    state.prefs(),
                );
            }
        }
        fbo.unbind(self.window_size);
        fbo
    }

    fn draw_error_paragraph(
        &self,
        resources: &Resources,
        fbo_size: RectSize<f32>,
        error: &str,
        prefs: &Prefs,
    ) {
        let matrix = cgmath::ortho(
            0.0,
            fbo_size.width,
            0.0,
            fbo_size.height,
            -10.0,
            10.0,
        );
        let paragraph = Paragraph::compile(
            DESCRIPTION_FONT_SIZE,
            DESCRIPTION_LINE_HEIGHT,
            fbo_size.width,
            prefs,
            &format!("$R$*ERROR:$*$D {}", Paragraph::escape(error)),
        );
        let top = (0.5 * (fbo_size.height - paragraph.height())).round();
        paragraph.draw(resources, &matrix, (0.0, top));
    }

    fn draw_edit_grid(
        &self,
        resources: &Resources,
        fbo_size: RectSize<f32>,
        grid: &EditGrid,
    ) {
        let grid_matrix = self.grid_matrix(fbo_size, grid);
        let board_rect = grid.bounds().as_f32().expand(0.25);
        resources.shaders().solid().fill_rect(
            &grid_matrix,
            Color3::PURPLE1,
            board_rect,
        );
        for (coords, dir, shape, size, color, error) in grid.wire_fragments() {
            let color = if error { WireColor::Ambiguous } else { color };
            WireModel::draw_fragment(
                resources,
                &grid_matrix,
                coords,
                dir,
                shape,
                color,
                size,
                &Color4::TRANSPARENT,
            );
        }
        for interface in grid.interfaces() {
            let coords = interface.top_left(grid.bounds());
            ChipModel::draw_interface(
                resources,
                &grid_matrix,
                coords,
                interface,
            );
        }
        for (coords, ctype, orient) in grid.chips() {
            ChipModel::draw_chip(
                resources,
                &grid_matrix,
                coords,
                ctype,
                orient,
                None,
            );
        }
    }

    fn grid_matrix(
        &self,
        fbo_size: RectSize<f32>,
        grid: &EditGrid,
    ) -> Matrix4<f32> {
        let board_bounds = grid.bounds().as_f32().expand(1.0);
        let board_aspect_ratio = board_bounds.width / board_bounds.height;
        let fbo_aspect_ratio = fbo_size.width / fbo_size.height;
        let (grid_width, grid_height) =
            if board_aspect_ratio > fbo_aspect_ratio {
                let min_width = fbo_size.width / PREVIEW_MAX_GRID_CELL_SIZE;
                let grid_width = board_bounds.width.max(min_width);
                (grid_width, grid_width / fbo_aspect_ratio)
            } else {
                let min_height = fbo_size.height / PREVIEW_MAX_GRID_CELL_SIZE;
                let grid_height = board_bounds.height.max(min_height);
                (grid_height * fbo_aspect_ratio, grid_height)
            };
        let grid_x = board_bounds.x - 0.5 * (grid_width - board_bounds.width);
        let grid_y =
            board_bounds.y - 0.5 * (grid_height - board_bounds.height);
        cgmath::ortho(
            grid_x,
            grid_x + grid_width,
            grid_y,
            grid_y + grid_height,
            -10.0,
            10.0,
        )
    }
}

//===========================================================================//
