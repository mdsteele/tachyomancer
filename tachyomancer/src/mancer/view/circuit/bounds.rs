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

use super::super::chip::CHIP_MARGIN;
use crate::mancer::gui::{Cursor, NextCursor, Sound, Ui};
use cgmath::Point2;
use tachy::geom::{
    AsFloat, AsInt, CoordsDelta, CoordsRect, CoordsSize, Direction, Rect,
};
use tachy::state::{EditGrid, GridChange};

//===========================================================================//

// The thickness, in grid cells, of the bounds margin:
pub const BOUNDS_MARGIN: f32 = 0.65;

//===========================================================================//

#[derive(Clone, Copy)]
pub enum BoundsHandle {
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
    Top,
    TopRight,
}

impl BoundsHandle {
    pub fn for_side(side: Direction) -> BoundsHandle {
        match side {
            Direction::East => BoundsHandle::Right,
            Direction::South => BoundsHandle::Bottom,
            Direction::West => BoundsHandle::Left,
            Direction::North => BoundsHandle::Top,
        }
    }

    pub fn for_grid_pt(
        grid_pt: Point2<f32>,
        grid: &EditGrid,
    ) -> Option<BoundsHandle> {
        let inner = grid.bounds().as_f32().expand(CHIP_MARGIN);
        if inner.contains_point(grid_pt) {
            return None;
        }
        let outer = inner.expand(BOUNDS_MARGIN);
        if !outer.contains_point(grid_pt) {
            return None;
        }
        let at_top = grid_pt.y < inner.y;
        let at_bottom = grid_pt.y >= inner.bottom();
        if grid_pt.x < inner.x {
            if at_top {
                Some(BoundsHandle::TopLeft)
            } else if at_bottom {
                Some(BoundsHandle::BottomLeft)
            } else {
                Some(BoundsHandle::Left)
            }
        } else if grid_pt.x >= inner.right() {
            if at_top {
                Some(BoundsHandle::TopRight)
            } else if at_bottom {
                Some(BoundsHandle::BottomRight)
            } else {
                Some(BoundsHandle::Right)
            }
        } else if at_top {
            Some(BoundsHandle::Top)
        } else if at_bottom {
            Some(BoundsHandle::Bottom)
        } else {
            None
        }
    }

    pub fn cursor(self) -> Cursor {
        match self {
            BoundsHandle::Right | BoundsHandle::Left => Cursor::ResizeEastWest,
            BoundsHandle::Top | BoundsHandle::Bottom => {
                Cursor::ResizeNorthSouth
            }
            BoundsHandle::TopRight | BoundsHandle::BottomLeft => {
                Cursor::ResizeNortheastSouthwest
            }
            BoundsHandle::TopLeft | BoundsHandle::BottomRight => {
                Cursor::ResizeNorthwestSoutheast
            }
        }
    }
}

//===========================================================================//

pub struct BoundsDrag {
    min_size: CoordsSize,
    handle: BoundsHandle,
    drag_start_grid_pt: Point2<f32>,
    drag_current_grid_pt: Point2<f32>,
    bounds: CoordsRect,
    acceptable: bool,
}

impl BoundsDrag {
    pub fn new(
        handle: BoundsHandle,
        start_grid_pt: Point2<f32>,
        grid: &mut EditGrid,
    ) -> BoundsDrag {
        BoundsDrag {
            min_size: grid.min_bounds_size(),
            handle,
            drag_start_grid_pt: start_grid_pt,
            drag_current_grid_pt: start_grid_pt,
            bounds: grid.bounds(),
            acceptable: true,
        }
    }

    pub fn bounds(&self) -> CoordsRect {
        self.bounds
    }

    pub fn is_acceptable(&self) -> bool {
        self.acceptable
    }

    pub fn request_cursor(&self, next_cursor: &mut NextCursor) {
        next_cursor.request(self.handle.cursor());
    }

    pub fn move_to(
        &mut self,
        grid_pt: Point2<f32>,
        ui: &mut Ui,
        grid: &EditGrid,
    ) {
        self.drag_current_grid_pt = grid_pt;
        let delta: CoordsDelta = (self.drag_current_grid_pt
            - self.drag_start_grid_pt)
            .as_i32_round();
        let old_bounds = grid.bounds();
        let mut left = old_bounds.x;
        let mut right = old_bounds.x + old_bounds.width;
        match self.handle {
            BoundsHandle::TopLeft
            | BoundsHandle::Left
            | BoundsHandle::BottomLeft => {
                left = (left + delta.x).min(right - self.min_size.width);
            }
            BoundsHandle::TopRight
            | BoundsHandle::Right
            | BoundsHandle::BottomRight => {
                right = (right + delta.x).max(left + self.min_size.width);
            }
            BoundsHandle::Top | BoundsHandle::Bottom => {}
        }
        let mut top = old_bounds.y;
        let mut bottom = old_bounds.y + old_bounds.height;
        match self.handle {
            BoundsHandle::TopLeft
            | BoundsHandle::Top
            | BoundsHandle::TopRight => {
                top = (top + delta.y).min(bottom - self.min_size.height);
            }
            BoundsHandle::BottomLeft
            | BoundsHandle::Bottom
            | BoundsHandle::BottomRight => {
                bottom = (bottom + delta.y).max(top + self.min_size.height);
            }
            BoundsHandle::Left | BoundsHandle::Right => {}
        }
        let new_bounds = Rect::new(left, top, right - left, bottom - top);
        if new_bounds != self.bounds {
            self.bounds = new_bounds;
            self.acceptable = grid.can_have_bounds(self.bounds);
            ui.request_redraw();
            ui.audio().play_sound(Sound::ChangeBounds);
        }
    }

    pub fn finish(self, ui: &mut Ui, grid: &mut EditGrid) {
        debug_assert_eq!(self.acceptable, grid.can_have_bounds(self.bounds));
        if self.acceptable {
            let old_bounds = grid.bounds();
            let changes = vec![GridChange::SetBounds(old_bounds, self.bounds)];
            if !grid.try_mutate(changes) {
                debug_warn!("BoundsDrag mutation failed");
            }
        } else {
            ui.audio().play_sound(Sound::ChangeBounds);
        }
        ui.request_redraw();
    }
}

//===========================================================================//
