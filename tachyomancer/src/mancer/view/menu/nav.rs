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

use super::super::paragraph::Paragraph;
use crate::mancer::gui::{Event, Resources, Ui};
use crate::mancer::save::Prefs;
use crate::mancer::state::GameState;
use cgmath::{vec2, Angle, Deg, Matrix4, Point2};
use std::f32;
use tachy::geom::{Color3, Color4, MatrixExt, Rect, RectSize};
use tachy::save::{Chapter, Puzzle};

//===========================================================================//

const LINE_COLOR: Color3 = Color3::ORANGE4;
const LINE_CORNER_HORZ: f32 = 40.0;
const LINE_CORNER_VERT: f32 = 24.0;
const LINE_THICKNESS: f32 = 4.0;
const PARAGRAPH_FONT_SIZE: f32 = 20.0;
const PARAGRAPH_LINE_HEIGHT: f32 = 22.0;
const PARAGRAPH_MARGIN_HORZ: f32 = 6.0;
const PARAGRAPH_MARGIN_VERT: f32 = 4.0;
const PARAGRAPH_MAX_WIDTH: f32 = 400.0;

//===========================================================================//

pub struct NavigationView {
    screen_size: RectSize<f32>,
    indicators: Vec<Indicator>,
}

impl NavigationView {
    pub fn new(
        screen_size: RectSize<f32>,
        state: &GameState,
        chapter: Chapter,
    ) -> NavigationView {
        let mut view = NavigationView { screen_size, indicators: Vec::new() };
        view.refresh_indicators(state, chapter);
        view
    }

    pub fn refresh_indicators(&mut self, state: &GameState, chapter: Chapter) {
        self.indicators.clear();
        match chapter {
            Chapter::Odyssey => self.refresh_odyssey(state),
            // TODO: other chapter backgrounds
            _ => {}
        }
    }

    fn refresh_odyssey(&mut self, state: &GameState) {
        self.add_indicator(
            (-0.91, 0.25),
            (90.0, -60.0),
            state.prefs(),
            "$C[Planet designation unknown]\n\
             Classification: $Y$*M$* (habitable)",
        );
        self.add_indicator(
            (0.86, -0.09),
            (20.0, -50.0),
            state.prefs(),
            &format!(
                "$C $/$*H.L.S. Odyssey$*$/\n    \
                 LTF: $ROFFLINE$C\n \
                 Backup: {}\n\
                 Sensors: {}\n  \
                 Orbit: $GSTABLE",
                if state.is_puzzle_solved(Puzzle::AutomateReactor) {
                    "$GONLINE$C"
                } else {
                    "$ROFFLINE$C"
                },
                if state.is_puzzle_solved(Puzzle::AutomateSensors) {
                    "$GONLINE$C"
                } else {
                    "$ROFFLINE$C"
                },
            ),
        );
        self.add_indicator(
            (0.77, -0.18),
            (-100.0, 30.0),
            state.prefs(),
            &format!(
                "$CSolar panels\n{}",
                if state.is_puzzle_solved(Puzzle::AutomateHeliostat) {
                    "$G(functional)"
                } else {
                    "    $R(jammed)"
                },
            ),
        );
        self.add_indicator(
            (0.87, -0.34),
            (-40.0, 100.0),
            state.prefs(),
            "$CHyperfuel tank\n    \
             $R(ruptured)",
        );
    }

    fn add_indicator(
        &mut self,
        tip: (f32, f32),
        offset: (f32, f32),
        prefs: &Prefs,
        format: &str,
    ) {
        let half_height = 0.5 * self.screen_size.height;
        let line_tip = Point2::new(
            (0.5 * self.screen_size.width + half_height * tip.0).round(),
            (half_height * (1.0 - tip.1)).round(),
        );
        let offset = vec2(offset.0, offset.1);
        let line_end = line_tip + offset;
        let paragraph = Paragraph::compile(
            PARAGRAPH_FONT_SIZE,
            PARAGRAPH_LINE_HEIGHT,
            PARAGRAPH_MAX_WIDTH,
            prefs,
            format,
        );
        let x_offset = if offset.x < 0.0 {
            -(paragraph.width() + PARAGRAPH_MARGIN_HORZ + LINE_THICKNESS * 0.5)
        } else {
            LINE_THICKNESS * 0.5 + PARAGRAPH_MARGIN_HORZ
        };
        let y_offset = if offset.y < 0.0 {
            -(paragraph.height()
                + PARAGRAPH_MARGIN_VERT
                + LINE_THICKNESS * 0.5)
        } else {
            LINE_THICKNESS * 0.5 + PARAGRAPH_MARGIN_VERT
        };
        let paragraph_top_left = line_end + vec2(x_offset, y_offset);
        self.indicators.push(Indicator {
            paragraph,
            paragraph_top_left,
            line_tip,
            line_end,
        });
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        for indicator in self.indicators.iter() {
            indicator.draw(resources, matrix);
        }
    }

    pub fn on_event(&mut self, event: &Event, _ui: &mut Ui) {
        match event {
            Event::MouseDown(mouse) => {
                if cfg!(debug_assertions) {
                    let half_height = 0.5 * self.screen_size.height;
                    let x = ((mouse.pt.x as f32)
                        - 0.5 * self.screen_size.width)
                        / half_height;
                    let y = 1.0 - (mouse.pt.y as f32) / half_height;
                    debug_log!("Tip position: ({}, {})", x, y);
                }
            }
            _ => {}
        }
    }
}

//===========================================================================//

struct Indicator {
    paragraph: Paragraph,
    paragraph_top_left: Point2<f32>,
    line_tip: Point2<f32>,
    line_end: Point2<f32>,
}

impl Indicator {
    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        // Paragraph:
        resources.shaders().ui().draw_scroll_bar(
            matrix,
            &Rect::new(
                self.paragraph_top_left.x - (PARAGRAPH_MARGIN_HORZ + 4.0),
                self.paragraph_top_left.y - (PARAGRAPH_MARGIN_VERT + 4.0),
                self.paragraph.width() + (PARAGRAPH_MARGIN_HORZ + 4.0) * 2.0,
                self.paragraph.height() + (PARAGRAPH_MARGIN_VERT + 4.0) * 2.0,
            ),
            &Color4::ORANGE2,
            &Color4::PURPLE1,
            &Color4::PURPLE0_TRANSLUCENT,
        );
        self.paragraph.draw(
            resources,
            matrix,
            (self.paragraph_top_left.x, self.paragraph_top_left.y),
        );
        // Diagonal line:
        let dx = self.line_end.x - self.line_tip.x;
        let dy = self.line_end.y - self.line_tip.y;
        let miter = LINE_THICKNESS * 0.5 * Deg(22.5).tan();
        let diagonal_length =
            dx.abs().min(dy.abs()) * f32::consts::SQRT_2 + miter;
        let diagonal_angle = if dx < 0.0 {
            if dy < 0.0 {
                Deg(-135.0)
            } else {
                Deg(135.0)
            }
        } else {
            if dy < 0.0 {
                Deg(-45.0)
            } else {
                Deg(45.0)
            }
        };
        resources.shaders().solid().fill_rect(
            &(matrix
                * Matrix4::trans2(self.line_tip.x, self.line_tip.y)
                * Matrix4::from_angle_z(diagonal_angle)),
            LINE_COLOR,
            Rect::new(
                0.0,
                -0.5 * LINE_THICKNESS,
                diagonal_length,
                LINE_THICKNESS,
            ),
        );
        // Horz/vert lines:
        if dx.abs() > dy.abs() {
            let length = dx.abs() - dy.abs() + miter;
            resources.shaders().solid().fill_rect(
                matrix,
                LINE_COLOR,
                Rect::new(
                    if dx < 0.0 {
                        self.line_end.x - LINE_CORNER_HORZ
                    } else {
                        self.line_end.x - length
                    },
                    self.line_end.y - 0.5 * LINE_THICKNESS,
                    length + LINE_CORNER_HORZ,
                    LINE_THICKNESS,
                ),
            );
            resources.shaders().solid().fill_rect(
                matrix,
                LINE_COLOR,
                Rect::new(
                    self.line_end.x - 0.5 * LINE_THICKNESS,
                    if dy < 0.0 {
                        self.line_end.y - LINE_CORNER_VERT
                    } else {
                        self.line_end.y
                    },
                    LINE_THICKNESS,
                    LINE_CORNER_VERT,
                ),
            );
        } else {
            let length = dy.abs() - dx.abs() + miter;
            resources.shaders().solid().fill_rect(
                matrix,
                LINE_COLOR,
                Rect::new(
                    self.line_end.x - 0.5 * LINE_THICKNESS,
                    if dy < 0.0 {
                        self.line_end.y - LINE_CORNER_VERT
                    } else {
                        self.line_end.y - length
                    },
                    LINE_THICKNESS,
                    length + LINE_CORNER_VERT,
                ),
            );
            resources.shaders().solid().fill_rect(
                matrix,
                LINE_COLOR,
                Rect::new(
                    if dx < 0.0 {
                        self.line_end.x - LINE_CORNER_HORZ
                    } else {
                        self.line_end.x
                    },
                    self.line_end.y - 0.5 * LINE_THICKNESS,
                    LINE_CORNER_HORZ,
                    LINE_THICKNESS,
                ),
            );
        }
    }
}

//===========================================================================//
