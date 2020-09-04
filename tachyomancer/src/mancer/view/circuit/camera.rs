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

use crate::mancer::gui::{ClockEventData, Ui};
use crate::mancer::save::{Hotkey, HotkeyCodeExt, Prefs};
use cgmath::{self, vec2, Matrix4, Point2, Vector2};
use tachy::geom::{AsFloat, AsInt, Coords, CoordsRect, MatrixExt, RectSize};

//===========================================================================//

// The size, in screen pixels, of a grid cell at 1x zoom:
pub const GRID_CELL_SIZE: i32 = 64;

// How far we scroll per second while holding down a scroll hotkey, in grid
// cells, at default zoom:
const SCROLL_GRID_CELLS_PER_SECOND: f64 = 12.0;

// The default zoom multiplier:
const ZOOM_DEFAULT: f32 = 1.0;
// The minimum zoom multiplier (i.e. how far zoomed out you can be):
const ZOOM_MIN: f32 = 0.25;
// The maximum zoom multiplier (i.e. how far zoomed in you can be):
const ZOOM_MAX: f32 = 2.0;

//===========================================================================//

pub struct EditGridCamera {
    window_size: RectSize<f32>,
    scroll: Vector2<i32>,
    scroll_goal: Option<Vector2<i32>>,
    zoom: f32,
}

impl EditGridCamera {
    pub fn new(
        window_size: RectSize<i32>,
        init_circuit_bounds: CoordsRect,
    ) -> EditGridCamera {
        let pixel_bounds = init_circuit_bounds * GRID_CELL_SIZE;
        EditGridCamera {
            window_size: window_size.as_f32(),
            scroll: Vector2::new(
                pixel_bounds.x + pixel_bounds.width / 2,
                pixel_bounds.y + pixel_bounds.height / 2,
            ),
            scroll_goal: None,
            zoom: ZOOM_DEFAULT,
        }
    }

    /// Returns the current size of the camera view, in grid cells.
    pub fn grid_view_size(&self) -> RectSize<f32> {
        self.window_size / ((GRID_CELL_SIZE as f32) * self.zoom)
    }

    /// Returns the current center of the camera view, in grid coordinates.
    pub fn center_grid_pt(&self) -> Point2<f32> {
        Point2::new(0.0, 0.0) + self.scroll.as_f32() / (GRID_CELL_SIZE as f32)
    }

    /// Returns the current width of a grid cell on the screen, in pixels.
    pub fn grid_cell_size_in_pixels(&self) -> f32 {
        (GRID_CELL_SIZE as f32) * self.zoom
    }

    /// Sets a point, in grid coordinates, that the camera center should
    /// automatically scroll to.
    pub fn set_goal(&mut self, grid_pt: Point2<f32>) {
        self.scroll_goal = Some(
            (grid_pt * (GRID_CELL_SIZE as f32)).as_i32_round()
                - Point2::new(0, 0),
        );
    }

    pub fn scroll_by_screen_dist(&mut self, x: i32, y: i32, ui: &mut Ui) {
        self.scroll += (vec2(x, y).as_f32() / self.zoom).as_i32_round();
        ui.request_redraw();
    }

    pub fn zoom_by(&mut self, factor: f32, ui: &mut Ui) {
        if factor < 1.0 {
            let minimum =
                if self.zoom > ZOOM_DEFAULT { ZOOM_DEFAULT } else { ZOOM_MIN };
            self.zoom = (self.zoom * factor).max(minimum);
            ui.request_redraw();
        } else if factor > 1.0 {
            let maximum =
                if self.zoom < ZOOM_DEFAULT { ZOOM_DEFAULT } else { ZOOM_MAX };
            self.zoom = (self.zoom * factor).min(maximum);
            ui.request_redraw();
        }
    }

    pub fn reset_zoom_to_default(&mut self, ui: &mut Ui) {
        if self.zoom != ZOOM_DEFAULT {
            self.zoom = ZOOM_DEFAULT;
            ui.request_redraw();
        }
    }

    fn ortho_matrix(&self) -> Matrix4<f32> {
        cgmath::ortho(
            -0.5 * self.window_size.width,
            0.5 * self.window_size.width,
            0.5 * self.window_size.height,
            -0.5 * self.window_size.height,
            -100.0,
            100.0,
        )
    }

    /// Returns a matrix that maps from circuit grid space to GL clip space.
    /// (In circuit grid space, the origin is at the top-left corner of grid
    /// cell (0, 0), and one unit of distance maps to the width of a grid
    /// cell.)  This matrix is suitable for drawing chips, wires, and other
    /// circuit elements.
    pub fn grid_matrix(&self) -> Matrix4<f32> {
        self.ortho_matrix()
            * Matrix4::from_scale(self.zoom)
            * Matrix4::trans2(-self.scroll.x as f32, -self.scroll.y as f32)
            * Matrix4::from_scale(GRID_CELL_SIZE as f32)
    }

    /// Returns a matrix that maps from "circuit UI space" to GL clip space.
    /// (In circuit UI space, the origin is at the top-left corner of grid cell
    /// (0, 0), and one unit of distance maps to one pixel on the screen.)
    /// This matrix is suitable for drawing UI elements that scroll along with
    /// the circuit, but don't zoom along with the circuit (such as tutorial
    /// bubbles).
    pub fn unzoomed_matrix(&self) -> Matrix4<f32> {
        self.ortho_matrix()
            * Matrix4::trans2(
                (-self.scroll.x as f32) * self.zoom,
                (-self.scroll.y as f32) * self.zoom,
            )
    }

    pub fn screen_pt_to_grid_pt(&self, screen_pt: Point2<i32>) -> Point2<f32> {
        let half_size = self.window_size * 0.5;
        let relative_to_center =
            screen_pt.as_f32() - vec2(half_size.width, half_size.height);
        let zoomed = relative_to_center / self.zoom;
        let scrolled = zoomed.as_i32_round() + self.scroll;
        scrolled.as_f32() / (GRID_CELL_SIZE as f32)
    }

    pub fn grid_pt_to_screen_pt(&self, grid_pt: Point2<f32>) -> Point2<i32> {
        let scrolled = (grid_pt * (GRID_CELL_SIZE as f32)).as_i32_round();
        let zoomed = (scrolled - self.scroll).as_f32();
        let relative_to_center = zoomed * self.zoom;
        let half_size = self.window_size * 0.5;
        (relative_to_center + vec2(half_size.width, half_size.height))
            .as_i32_round()
    }

    pub fn coords_for_screen_pt(&self, screen_pt: Point2<i32>) -> Coords {
        self.screen_pt_to_grid_pt(screen_pt).as_i32_floor()
    }

    pub fn on_clock_tick(
        &mut self,
        tick: &ClockEventData,
        ui: &mut Ui,
        bounds: CoordsRect,
        prefs: &Prefs,
    ) {
        if let Some(goal) = self.scroll_goal {
            if self.scroll != goal {
                self.scroll.x = track_towards(self.scroll.x, goal.x, tick);
                self.scroll.y = track_towards(self.scroll.y, goal.y, tick);
                ui.request_redraw();
            }
            if self.scroll == goal {
                self.scroll_goal = None;
            } else {
                return;
            }
        }
        // Scroll if we're holding down any scroll key(s):
        let left = is_hotkey_held(Hotkey::ScrollLeft, ui, prefs);
        let right = is_hotkey_held(Hotkey::ScrollRight, ui, prefs);
        let up = is_hotkey_held(Hotkey::ScrollUp, ui, prefs);
        let down = is_hotkey_held(Hotkey::ScrollDown, ui, prefs);
        let dist = ((SCROLL_GRID_CELLS_PER_SECOND * tick.elapsed)
            * (GRID_CELL_SIZE as f64))
            .round() as i32;
        if left && !right {
            self.scroll_by_screen_dist(-dist, 0, ui);
        } else if right && !left {
            self.scroll_by_screen_dist(dist, 0, ui);
        }
        if up && !down {
            self.scroll_by_screen_dist(0, -dist, ui);
        } else if down && !up {
            self.scroll_by_screen_dist(0, dist, ui);
        }
        // Spring back to scroll bounds:
        let expand = (self.window_size * (0.25 / self.zoom)).as_i32_round();
        let scroll_limit =
            (bounds * GRID_CELL_SIZE).expand2(expand.width, expand.height);
        if self.scroll.x < scroll_limit.x {
            self.scroll.x = track_towards(self.scroll.x, scroll_limit.x, tick);
            ui.request_redraw();
        } else if self.scroll.x > scroll_limit.right() {
            self.scroll.x =
                track_towards(self.scroll.x, scroll_limit.right(), tick);
            ui.request_redraw();
        }
        if self.scroll.y < scroll_limit.y {
            self.scroll.y = track_towards(self.scroll.y, scroll_limit.y, tick);
            ui.request_redraw();
        } else if self.scroll.y > scroll_limit.bottom() {
            self.scroll.y =
                track_towards(self.scroll.y, scroll_limit.bottom(), tick);
            ui.request_redraw();
        }
    }
}

//===========================================================================//

fn is_hotkey_held(hotkey: Hotkey, ui: &Ui, prefs: &Prefs) -> bool {
    ui.keyboard().is_held(prefs.hotkey_code(hotkey).to_keycode())
}

fn track_towards(current: i32, goal: i32, tick: &ClockEventData) -> i32 {
    let tracking_base: f64 = 0.0001; // smaller = faster tracking
    let difference = (goal - current) as f64;
    let change = difference * (1.0 - tracking_base.powf(tick.elapsed));
    let change = change.round() as i32;
    if change != 0 {
        current + change
    } else if goal > current {
        current + 1
    } else if goal < current {
        current - 1
    } else {
        goal
    }
}

//===========================================================================//
