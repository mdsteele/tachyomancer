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

use crate::mancer::font::Align;
use crate::mancer::gl::FrameBuffer;
use crate::mancer::gui::Resources;
use crate::mancer::state::GameState;
use cgmath::{vec2, Deg, Matrix4};
use std::cell::RefCell;
use tachy::geom::{AsFloat, Color3, Color4, MatrixExt, Rect, RectSize};
use tachy::save::{Puzzle, PuzzleKind, ScoreCurve};

//===========================================================================//

const AXIS_THICKNESS: f32 = 2.0;
const CURVE_THICKNESS: f32 = 2.0;
const INNER_MARGIN: f32 = 12.0;
const AXIS_LABEL_FONT_SIZE: f32 = 18.0;
const LABEL_MARGIN: f32 = 26.0;
const TICK_LENGTH: f32 = 4.0;
const TICK_THICKNESS: f32 = 2.0;
const TICK_LABEL_FONT_SIZE: f32 = 16.0;

fn tick_step_for_maximum(max: i32) -> i32 {
    if max >= 4000 {
        1000
    } else if max >= 1000 {
        250
    } else if max >= 400 {
        100
    } else if max >= 100 {
        25
    } else {
        10
    }
}

fn format_tick_maximum(max: i32) -> String {
    if max >= 1000 && max % 1000 == 0 {
        format!("{}k", max / 1000)
    } else {
        format!("{}", max)
    }
}

//===========================================================================//

pub struct ScoreGraphView {
    window_size: RectSize<i32>,
    rect: Rect<f32>,
    cache: RefCell<Option<(Option<String>, Puzzle, FrameBuffer)>>,
}

impl ScoreGraphView {
    pub fn new(window_size: RectSize<i32>, rect: Rect<i32>) -> ScoreGraphView {
        ScoreGraphView {
            window_size,
            rect: rect.as_f32(),
            cache: RefCell::new(None),
        }
    }

    pub fn clear_cache(&mut self) {
        *self.cache.borrow_mut() = None;
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        state: &GameState,
    ) {
        let profile_name = state.profile().map(|profile| profile.name());
        let puzzle = state.current_puzzle();

        resources.shaders().ui().draw_scroll_bar(
            matrix,
            &self.rect,
            &Color4::ORANGE2,
            &Color4::PURPLE1,
            &Color4::PURPLE0_TRANSLUCENT,
        );

        let mut cached = self.cache.borrow_mut();
        match cached.as_ref() {
            Some(&(ref pname, puzz, ref fbo))
                if pname.as_ref().map(String::as_str) == profile_name
                    && puzz == puzzle =>
            {
                self.draw_fbo(resources, matrix, fbo);
                return;
            }
            _ => {}
        }

        let fbo =
            self.generate_fbo(resources, puzzle, state.local_scores(puzzle));
        self.draw_fbo(resources, matrix, &fbo);
        *cached = Some((profile_name.map(str::to_string), puzzle, fbo));
    }

    fn draw_fbo(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        fbo: &FrameBuffer,
    ) {
        let left_top = self.rect.top_left() + vec2(INNER_MARGIN, INNER_MARGIN);
        let grayscale = false;
        resources.shaders().frame().draw(matrix, fbo, left_top, grayscale);
    }

    fn generate_fbo(
        &self,
        resources: &Resources,
        puzzle: Puzzle,
        local_scores: &ScoreCurve,
    ) -> FrameBuffer {
        debug_log!("Regenerating preview image");
        let fbo_size = self.rect.size().expand(-INNER_MARGIN);
        let fbo = FrameBuffer::new(
            fbo_size.width as usize,
            fbo_size.height as usize,
            false,
        );
        fbo.bind();
        if puzzle.kind() == PuzzleKind::Sandbox {
            ScoreGraphView::draw_sandbox_message(resources, fbo_size);
        } else if !local_scores.is_empty() {
            ScoreGraphView::draw_graph(
                resources,
                fbo_size,
                puzzle,
                local_scores,
            );
        } else {
            ScoreGraphView::draw_no_solutions_message(resources, fbo_size);
        }
        fbo.unbind(self.window_size);
        fbo
    }

    fn draw_sandbox_message(resources: &Resources, fbo_size: RectSize<f32>) {
        let matrix = ScoreGraphView::fbo_matrix(fbo_size);
        let mid_x = (0.5 * fbo_size.width).round();
        let mid_y = (0.5 * fbo_size.height).round();
        let font = resources.fonts().roman();
        font.draw(
            &matrix,
            20.0,
            Align::MidCenter,
            (mid_x, mid_y),
            "NO GRAPH FOR SANDBOX",
        );
    }

    fn draw_no_solutions_message(
        resources: &Resources,
        fbo_size: RectSize<f32>,
    ) {
        let matrix = ScoreGraphView::fbo_matrix(fbo_size);
        let mid_x = (0.5 * fbo_size.width).round();
        let mid_y = (0.5 * fbo_size.height).round();
        let font = resources.fonts().roman();
        font.draw(
            &matrix,
            18.0,
            Align::MidCenter,
            (mid_x, mid_y - 12.0),
            "COMPLETE THIS TASK TO",
        );
        font.draw(
            &matrix,
            18.0,
            Align::MidCenter,
            (mid_x, mid_y + 12.0),
            "VIEW OPTIMIZATION GRAPH",
        );
    }

    fn draw_graph(
        resources: &Resources,
        fbo_size: RectSize<f32>,
        puzzle: Puzzle,
        local_scores: &ScoreCurve,
    ) {
        let matrix = ScoreGraphView::fbo_matrix(fbo_size);

        // Draw the graph data:
        let graph_rect = Rect::new(
            LABEL_MARGIN,
            0.0,
            fbo_size.width - LABEL_MARGIN,
            fbo_size.height - LABEL_MARGIN,
        );
        let graph_bounds = puzzle.graph_bounds();
        let global_scores = resources.global_scores_for(puzzle);
        ScoreGraphView::draw_score_curve(
            resources,
            &matrix,
            graph_rect,
            graph_bounds,
            &global_scores,
            Color3::CYAN4,
            Color3::CYAN2,
        );
        ScoreGraphView::draw_score_curve(
            resources,
            &matrix,
            graph_rect,
            graph_bounds,
            local_scores,
            Color3::ORANGE4,
            Color3::ORANGE1,
        );

        // Draw axes:
        let color = Color3::PURPLE3;
        let axis_rect = Rect::new(
            graph_rect.x - 0.5 * AXIS_THICKNESS,
            graph_rect.y,
            AXIS_THICKNESS,
            graph_rect.height + 0.5 * AXIS_THICKNESS,
        );
        resources.shaders().solid().fill_rect(&matrix, color, axis_rect);
        let axis_rect = Rect::new(
            graph_rect.x - 0.5 * AXIS_THICKNESS,
            graph_rect.bottom() - 0.5 * AXIS_THICKNESS,
            graph_rect.width + 0.5 * AXIS_THICKNESS,
            AXIS_THICKNESS,
        );
        resources.shaders().solid().fill_rect(&matrix, color, axis_rect);

        // Draw axis ticks:
        let unit_span =
            (graph_rect.width - TICK_THICKNESS) / (graph_bounds.0 as f32);
        let tick_step = tick_step_for_maximum(graph_bounds.0);
        let mut tick = 0;
        while tick <= graph_bounds.0 {
            let tick_rect = Rect::new(
                graph_rect.x + (tick as f32) * unit_span,
                graph_rect.bottom(),
                TICK_THICKNESS,
                TICK_LENGTH,
            );
            resources.shaders().solid().fill_rect(&matrix, color, tick_rect);
            tick += tick_step;
        }
        let unit_span =
            (graph_rect.height - TICK_THICKNESS) / (graph_bounds.1 as f32);
        let tick_step = tick_step_for_maximum(graph_bounds.1 as i32);
        let mut tick = 0;
        while tick <= (graph_bounds.1 as i32) {
            let tick_rect = Rect::new(
                graph_rect.x - TICK_LENGTH,
                graph_rect.bottom()
                    - TICK_THICKNESS
                    - (tick as f32) * unit_span,
                TICK_LENGTH,
                TICK_THICKNESS,
            );
            resources.shaders().solid().fill_rect(&matrix, color, tick_rect);
            tick += tick_step;
        }

        // Draw tick labels:
        let font = resources.fonts().roman();
        font.draw(
            &matrix,
            TICK_LABEL_FONT_SIZE,
            Align::TopRight,
            (
                graph_rect.x - TICK_THICKNESS,
                graph_rect.bottom() + TICK_THICKNESS,
            ),
            "0",
        );
        font.draw(
            &matrix,
            TICK_LABEL_FONT_SIZE,
            Align::TopRight,
            (graph_rect.right() + 1.0, graph_rect.bottom() + TICK_LENGTH),
            &format_tick_maximum(graph_bounds.0),
        );
        font.draw(
            &matrix,
            TICK_LABEL_FONT_SIZE,
            Align::TopCenter,
            (
                graph_rect.x
                    - AXIS_THICKNESS
                    - 1.0
                    - 1.5 * TICK_LABEL_FONT_SIZE * font.ratio(),
                graph_rect.y,
            ),
            &format_tick_maximum(graph_bounds.1 as i32),
        );

        // Draw axis labels:
        font.draw(
            &matrix,
            AXIS_LABEL_FONT_SIZE,
            Align::BottomCenter,
            (
                graph_rect.x + 0.5 * graph_rect.width,
                graph_rect.bottom() + LABEL_MARGIN as f32,
            ),
            "Area",
        );
        let side_matrix = matrix
            * Matrix4::trans2(graph_rect.x, graph_rect.bottom())
            * Matrix4::from_angle_z(Deg(-90.0));
        font.draw(
            &side_matrix,
            AXIS_LABEL_FONT_SIZE,
            Align::TopCenter,
            (0.5 * graph_rect.height, -LABEL_MARGIN as f32),
            puzzle.score_units().label(),
        );
    }

    fn draw_score_curve(
        resources: &Resources,
        matrix: &Matrix4<f32>,
        graph_rect: Rect<f32>,
        graph_bounds: (i32, u32),
        scores: &ScoreCurve,
        line_color: Color3,
        fill_color: Color3,
    ) {
        let (max_area, max_score) = graph_bounds;
        let mut prev_rel_y = 0.0;
        for &(area, score) in scores.scores().iter() {
            let rel_x = graph_rect.width * ((area as f32) / (max_area as f32));
            let rel_y = graph_rect.height
                - graph_rect.height * ((score as f32) / (max_score as f32));
            let solid = resources.shaders().solid();
            let fill_rect = Rect::new(
                graph_rect.x + rel_x + 0.5 * CURVE_THICKNESS,
                graph_rect.y,
                graph_rect.width - rel_x - 0.5 * CURVE_THICKNESS,
                rel_y,
            );
            solid.fill_rect(&matrix, fill_color, fill_rect);
            let vert_rect = Rect::new(
                graph_rect.x + rel_x - 0.5 * CURVE_THICKNESS,
                graph_rect.y + prev_rel_y,
                CURVE_THICKNESS,
                rel_y - prev_rel_y,
            );
            solid.fill_rect(&matrix, line_color, vert_rect);
            let horz_rect = Rect::new(
                graph_rect.x + rel_x - 0.5 * CURVE_THICKNESS,
                graph_rect.y + rel_y - 0.5 * CURVE_THICKNESS,
                graph_rect.width - rel_x,
                CURVE_THICKNESS,
            );
            solid.fill_rect(&matrix, line_color, horz_rect);
            prev_rel_y = rel_y;
        }
    }

    fn fbo_matrix(fbo_size: RectSize<f32>) -> Matrix4<f32> {
        cgmath::ortho(0.0, fbo_size.width, 0.0, fbo_size.height, -10.0, 10.0)
    }
}

//===========================================================================//
