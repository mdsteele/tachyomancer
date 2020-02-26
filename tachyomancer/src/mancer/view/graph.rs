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
use crate::mancer::gl::FrameBufferMultisample;
use crate::mancer::gui::{Event, Resources, Ui};
use cgmath::{Deg, Matrix4, MetricSpace, Point2};
use std::cell::RefCell;
use tachy::geom::{AsFloat, Color3, MatrixExt, Rect};
use tachy::save::{Puzzle, ScoreCurve};

//===========================================================================//

const AXIS_THICKNESS: f32 = 2.0;
const CURVE_THICKNESS: f32 = 2.0;
const AXIS_LABEL_FONT_SIZE: f32 = 18.0;
const LABEL_MARGIN: f32 = 26.0;
const MAX_HILIGHT_DIST: f32 = 15.0;
const TICK_LENGTH: f32 = 4.0;
const TICK_THICKNESS: f32 = 2.0;
const TICK_LABEL_FONT_SIZE: f32 = 16.0;
const SCORE_HILIGHT_RADIUS: f32 = 4.0;

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

struct ScoreGraphCache {
    global_scores: ScoreCurve,
    fbo: FrameBufferMultisample,
}

pub struct ScoreGraph {
    rect: Rect<f32>,
    puzzle: Puzzle,
    local_scores: ScoreCurve,
    default_hilight: Option<(i32, u32)>,
    current_hilight: Option<(i32, u32)>,
    cache: RefCell<Option<ScoreGraphCache>>,
}

impl ScoreGraph {
    pub fn new(
        rect: Rect<f32>,
        puzzle: Puzzle,
        local_scores: &ScoreCurve,
        hilight_score: Option<(i32, u32)>,
    ) -> ScoreGraph {
        ScoreGraph {
            rect,
            puzzle,
            local_scores: local_scores.clone(),
            default_hilight: hilight_score,
            current_hilight: hilight_score,
            cache: RefCell::new(None),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let mut opt_cache = self.cache.borrow_mut();
        if let Some(cache) = opt_cache.as_ref() {
            self.draw_fbo(resources, matrix, &cache.fbo);
            return;
        }
        let global_scores = resources.global_scores_for(self.puzzle);
        let fbo = self.generate_fbo(resources, &global_scores);
        self.draw_fbo(resources, matrix, &fbo);
        *opt_cache = Some(ScoreGraphCache { global_scores, fbo });
    }

    fn draw_fbo(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        fbo: &FrameBufferMultisample,
    ) {
        let left_top = self.rect.top_left();
        let grayscale = false;
        resources.shaders().frame().draw(matrix, fbo, left_top, grayscale);

        // Draw hilighted score:
        if let Some(score) = self.current_hilight {
            let center = self.score_position(score);
            resources.shaders().solid().fill_rect(
                matrix,
                Color3::YELLOW5,
                Rect::new(
                    center.x - SCORE_HILIGHT_RADIUS,
                    center.y - SCORE_HILIGHT_RADIUS,
                    2.0 * SCORE_HILIGHT_RADIUS,
                    2.0 * SCORE_HILIGHT_RADIUS,
                ),
            );
        }
    }

    fn generate_fbo(
        &self,
        resources: &Resources,
        global_scores: &ScoreCurve,
    ) -> FrameBufferMultisample {
        debug_log!("Regenerating score graph image");
        let fbo_size = self.rect.size();
        let fbo = FrameBufferMultisample::new(
            fbo_size.width as usize,
            fbo_size.height as usize,
            false,
        );
        fbo.bind();
        let matrix = cgmath::ortho(
            0.0,
            fbo_size.width,
            0.0,
            fbo_size.height,
            -10.0,
            10.0,
        );
        let graph_rect = Rect::new(
            LABEL_MARGIN,
            0.0,
            fbo_size.width - LABEL_MARGIN,
            fbo_size.height - LABEL_MARGIN,
        );
        ScoreGraph::draw_graph(
            resources,
            &matrix,
            graph_rect,
            self.puzzle,
            &self.local_scores,
            global_scores,
        );
        fbo.unbind(resources.window_size());
        fbo
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui) {
        match event {
            Event::MouseMove(mouse) => {
                let mouse_pt = mouse.pt.as_f32();
                if !self.rect.expand(MAX_HILIGHT_DIST).contains_point(mouse_pt)
                {
                    return;
                }
                let mut closest = self.default_hilight;
                let mut closest_dist = MAX_HILIGHT_DIST;
                self.for_each_score(|score| {
                    let score_pt = self.score_position(score);
                    let dist = score_pt.distance(mouse_pt);
                    if dist < closest_dist {
                        closest = Some(score);
                        closest_dist = dist;
                    }
                });
                // TODO: tooltip for hovered score
                if closest != self.current_hilight {
                    self.current_hilight = closest;
                    ui.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn for_each_score<F>(&self, mut func: F)
    where
        F: FnMut((i32, u32)),
    {
        if let Some(score) = self.default_hilight {
            func(score);
        }
        for &score in self.local_scores.scores() {
            func(score);
        }
        let opt_cache = self.cache.borrow();
        if let Some(cache) = opt_cache.as_ref() {
            for &score in cache.global_scores.scores() {
                func(score);
            }
        }
    }

    fn score_position(&self, score: (i32, u32)) -> Point2<f32> {
        let graph_rect = Rect::new(
            self.rect.x + LABEL_MARGIN,
            self.rect.y,
            self.rect.width - LABEL_MARGIN,
            self.rect.height - LABEL_MARGIN,
        );
        let graph_bounds = self.puzzle.graph_bounds();
        let cx = graph_rect.x
            + graph_rect.width * ((score.0 as f32) / (graph_bounds.0 as f32));
        let cy = graph_rect.bottom()
            - graph_rect.height * ((score.1 as f32) / (graph_bounds.1 as f32));
        Point2::new(cx, cy)
    }

    fn draw_graph(
        resources: &Resources,
        matrix: &Matrix4<f32>,
        graph_rect: Rect<f32>,
        puzzle: Puzzle,
        local_scores: &ScoreCurve,
        global_scores: &ScoreCurve,
    ) {
        // Draw the graph data:
        let graph_bounds = puzzle.graph_bounds();
        ScoreGraph::draw_score_curve(
            resources,
            matrix,
            graph_rect,
            graph_bounds,
            global_scores,
            Color3::CYAN4,
            Color3::CYAN2,
        );
        ScoreGraph::draw_score_curve(
            resources,
            matrix,
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
        resources.shaders().solid().fill_rect(matrix, color, axis_rect);
        let axis_rect = Rect::new(
            graph_rect.x - 0.5 * AXIS_THICKNESS,
            graph_rect.bottom() - 0.5 * AXIS_THICKNESS,
            graph_rect.width + 0.5 * AXIS_THICKNESS,
            AXIS_THICKNESS,
        );
        resources.shaders().solid().fill_rect(matrix, color, axis_rect);

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
            resources.shaders().solid().fill_rect(matrix, color, tick_rect);
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
            resources.shaders().solid().fill_rect(matrix, color, tick_rect);
            tick += tick_step;
        }

        // Draw tick labels:
        let font = resources.fonts().roman();
        font.draw(
            matrix,
            TICK_LABEL_FONT_SIZE,
            Align::TopRight,
            (
                graph_rect.x - TICK_THICKNESS,
                graph_rect.bottom() + TICK_THICKNESS,
            ),
            "0",
        );
        font.draw(
            matrix,
            TICK_LABEL_FONT_SIZE,
            Align::TopRight,
            (graph_rect.right() + 1.0, graph_rect.bottom() + TICK_LENGTH),
            &format_tick_maximum(graph_bounds.0),
        );
        font.draw(
            matrix,
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
            matrix,
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
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::format_tick_maximum;

    #[test]
    fn max_tick_format() {
        assert_eq!(format_tick_maximum(100), "100");
        assert_eq!(format_tick_maximum(700), "700");
        assert_eq!(format_tick_maximum(1000), "1k");
        assert_eq!(format_tick_maximum(3000), "3k");
        assert_eq!(format_tick_maximum(3500), "3500");
    }
}

//===========================================================================//
