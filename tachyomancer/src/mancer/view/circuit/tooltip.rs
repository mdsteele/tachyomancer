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

use super::super::chip::{chip_grid_rect, interface_grid_rect};
use cgmath::Point2;
use tachy::geom::{AsInt, Coords, Direction, PolygonRef};
use tachy::save::{ChipType, WireShape};
use tachy::state::EditGrid;

//===========================================================================//

#[derive(Eq, PartialEq)]
pub enum GridTooltipTag {
    Chip(Coords, ChipType),
    Interface(usize),
    Wire(usize),
}

impl GridTooltipTag {
    pub fn for_grid_pt(
        grid: &EditGrid,
        grid_pt: Point2<f32>,
    ) -> Option<GridTooltipTag> {
        let coords: Coords = grid_pt.as_i32_floor();
        if let Some((coords, ctype, orient)) = grid.chip_at(coords) {
            if chip_grid_rect(coords, ctype, orient).contains_point(grid_pt) {
                return Some(GridTooltipTag::Chip(coords, ctype));
            }
        }
        if let Some((coords, index, iface)) = grid.interface_at(coords) {
            if interface_grid_rect(coords, iface).contains_point(grid_pt) {
                return Some(GridTooltipTag::Interface(index));
            }
        }

        let sub_pt = Point2::new(
            grid_pt.x - (coords.x as f32),
            grid_pt.y - (coords.y as f32),
        );
        let mut wire_dir: Option<Direction> = None;
        for dir in Direction::all() {
            let contains =
                match (grid.wire_shape_at(coords, dir), dir) {
                    (Some(WireShape::Stub), _) => {
                        STUB_POLYGON.contains_point(transform(sub_pt, dir))
                    }
                    (Some(WireShape::Straight), Direction::East)
                    | (Some(WireShape::Straight), Direction::North) => {
                        STRAIGHT_POLYGON.contains_point(transform(sub_pt, dir))
                    }
                    (Some(WireShape::TurnLeft), _) => TURN_LEFT_POLYGON
                        .contains_point(transform(sub_pt, dir)),
                    (Some(WireShape::SplitTee), _) => SPLIT_TEE_POLYGON
                        .contains_point(transform(sub_pt, dir)),
                    (Some(WireShape::Cross), Direction::East) => {
                        CROSS_POLYGON.contains_point(sub_pt)
                    }
                    _ => false,
                };
            if contains {
                wire_dir = Some(dir);
            }
        }
        if let Some(dir) = wire_dir {
            let index = grid.wire_index_at(coords, dir).unwrap();
            return Some(GridTooltipTag::Wire(index));
        }
        return None;
    }

    pub fn tooltip_format(&self, grid: &EditGrid) -> String {
        match *self {
            GridTooltipTag::Chip(_, ctype) => ctype.tooltip_format(),
            GridTooltipTag::Interface(index) => {
                grid.interfaces()[index].tooltip_format()
            }
            GridTooltipTag::Wire(index) => grid.wire_tooltip_format(index),
        }
    }
}

//===========================================================================//

// The cosine of 67.5 degrees:
const COS_67_5: f32 = 0.38268343236508984;

#[cfg_attr(rustfmt, rustfmt_skip)]
const STUB_POLYGON: PolygonRef = PolygonRef::new(&[
    Point2 { x: 1.1, y: 0.25 },
    Point2 { x: 60./64., y: 0.25 },
    Point2 { x: 56./64., y: 0.375 },
    Point2 { x: 56./64., y: 0.625 },
    Point2 { x: 60./64., y: 0.75 },
    Point2 { x: 1.1, y: 0.75 },
]);

#[cfg_attr(rustfmt, rustfmt_skip)]
const STRAIGHT_POLYGON: PolygonRef = PolygonRef::new(&[
    Point2 { x: 1.1, y: 0.25 },
    Point2 { x: -0.1, y: 0.25 },
    Point2 { x: -0.1, y: 0.75 },
    Point2 { x: 1.1, y: 0.75 },
]);

#[cfg_attr(rustfmt, rustfmt_skip)]
const TURN_LEFT_POLYGON: PolygonRef = PolygonRef::new(&[
    Point2 { x: 1.1, y: 0.25 },
    Point2 { x: 57./64. - 0.25 * COS_67_5, y: 0.25 },
    Point2 { x: 0.25, y: 57./64. - 0.25 * COS_67_5 },
    Point2 { x: 0.25, y: 1.1 },
    Point2 { x: 0.75, y: 1.1 },
    Point2 { x: 0.75, y: 57./64. + 0.25 * COS_67_5 },
    Point2 { x: 57./64. + 0.25 * COS_67_5, y: 0.75 },
    Point2 { x: 1.1, y: 0.75 },
]);

#[cfg_attr(rustfmt, rustfmt_skip)]
const SPLIT_TEE_POLYGON: PolygonRef = PolygonRef::new(&[
    Point2 { x: 1.1, y: 0.25 },
    Point2 { x: 0.75, y: 0.25 },
    Point2 { x: 0.75, y: -0.1 },
    Point2 { x: 0.25, y: -0.1 },
    Point2 { x: 0.25, y: 1.1 },
    Point2 { x: 0.75, y: 1.1 },
    Point2 { x: 0.75, y: 0.75 },
    Point2 { x: 1.1, y: 0.75 },
]);

#[cfg_attr(rustfmt, rustfmt_skip)]
const CROSS_POLYGON: PolygonRef = PolygonRef::new(&[
    Point2 { x: 1.1, y: 0.25 },
    Point2 { x: 0.75, y: 0.25 },
    Point2 { x: 0.75, y: -0.1 },
    Point2 { x: 0.25, y: -0.1 },
    Point2 { x: 0.25, y: 0.25 },
    Point2 { x: -0.1, y: 0.25 },
    Point2 { x: -0.1, y: 0.75 },
    Point2 { x: 0.25, y: 0.75 },
    Point2 { x: 0.25, y: 1.1 },
    Point2 { x: 0.75, y: 1.1 },
    Point2 { x: 0.75, y: 0.75 },
    Point2 { x: 1.1, y: 0.75 },
]);

fn transform(sub_pt: Point2<f32>, dir: Direction) -> Point2<f32> {
    match dir {
        Direction::East => sub_pt,
        Direction::South => Point2::new(sub_pt.y, 1.0 - sub_pt.x),
        Direction::West => Point2::new(1.0 - sub_pt.x, 1.0 - sub_pt.y),
        Direction::North => Point2::new(1.0 - sub_pt.y, sub_pt.x),
    }
}

//===========================================================================//
