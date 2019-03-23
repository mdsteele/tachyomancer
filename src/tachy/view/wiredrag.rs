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

use cgmath::Point2;
use tachy::geom::{AsInt, Coords, Direction};
use tachy::save::WireShape;
use tachy::state::{EditGrid, GridChange};

//===========================================================================//

const ZONE_CENTER_SEMI_SIZE: f32 = 0.1875;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Zone {
    Center(Coords),
    East(Coords),
    South(Coords),
}

impl Zone {
    pub fn from_grid_pt(grid_pt: Point2<f32>) -> Zone {
        let coords: Coords = grid_pt.as_i32_floor();
        let x = grid_pt.x - (coords.x as f32) - 0.5;
        let y = grid_pt.y - (coords.y as f32) - 0.5;
        if x.abs() <= ZONE_CENTER_SEMI_SIZE &&
            y.abs() <= ZONE_CENTER_SEMI_SIZE
        {
            Zone::Center(coords)
        } else if x.abs() > y.abs() {
            Zone::East(if x > 0.0 {
                           coords
                       } else {
                           coords + Direction::West
                       })
        } else {
            Zone::South(if y > 0.0 {
                            coords
                        } else {
                            coords + Direction::North
                        })
        }
    }
}

//===========================================================================//

pub struct WireDrag {
    curr: Option<Zone>,
    prev: Option<Zone>,
    changed: bool,
}

// TODO: enforce wires must be in bounds
// TODO: enforce wires can't be created under chips
impl WireDrag {
    pub fn new() -> WireDrag {
        WireDrag {
            curr: None,
            prev: None,
            changed: false,
        }
    }

    pub fn move_to(&mut self, zone: Zone, grid: &mut EditGrid) -> bool {
        if self.curr == Some(zone) {
            return true;
        }
        let more = match (self.prev, self.curr, zone) {
            (_, None, Zone::East(coords)) => {
                self.try_start_stub(coords, Direction::East, grid)
            }
            (_, None, Zone::South(coords)) => {
                self.try_start_stub(coords, Direction::South, grid)
            }
            (None, Some(Zone::Center(coords1)), Zone::East(coords2)) => {
                if coords1 == coords2 {
                    self.try_split(coords1, Direction::East, grid)
                } else if coords1 + Direction::West == coords2 {
                    self.try_split(coords1, Direction::West, grid)
                } else {
                    true
                }
            }
            (None, Some(Zone::Center(coords1)), Zone::South(coords2)) => {
                if coords1 == coords2 {
                    self.try_split(coords1, Direction::South, grid)
                } else if coords1 + Direction::North == coords2 {
                    self.try_split(coords1, Direction::North, grid)
                } else {
                    true
                }
            }
            (Some(Zone::East(coords1)), _, Zone::East(coords2)) => {
                if coords1 + Direction::East == coords2 {
                    self.try_straight(coords2, Direction::East, grid)
                } else if coords1 + Direction::West == coords2 {
                    self.try_straight(coords1, Direction::West, grid)
                } else {
                    true
                }
            }
            (Some(Zone::South(coords1)), _, Zone::South(coords2)) => {
                if coords1 + Direction::South == coords2 {
                    self.try_straight(coords2, Direction::South, grid)
                } else if coords1 + Direction::North == coords2 {
                    self.try_straight(coords1, Direction::North, grid)
                } else {
                    true
                }
            }
            (_, Some(Zone::East(coords1)), Zone::South(coords2)) => {
                if coords1 == coords2 {
                    self.try_turn_left(coords1, Direction::East, grid)
                } else if coords1 + Direction::North == coords2 {
                    self.try_turn_left(coords1, Direction::North, grid)
                } else if coords1 + Direction::East == coords2 {
                    self.try_turn_left(coords2, Direction::South, grid)
                } else if coords1 + Direction::East ==
                           coords2 + Direction::South
                {
                    self.try_turn_left(coords1 + Direction::East,
                                       Direction::West,
                                       grid)
                } else {
                    true
                }
            }
            (_, Some(Zone::South(coords1)), Zone::East(coords2)) => {
                if coords1 == coords2 {
                    self.try_turn_left(coords1, Direction::East, grid)
                } else if coords1 + Direction::South == coords2 {
                    self.try_turn_left(coords2, Direction::North, grid)
                } else if coords1 + Direction::West == coords2 {
                    self.try_turn_left(coords1, Direction::South, grid)
                } else if coords1 + Direction::South ==
                           coords2 + Direction::East
                {
                    self.try_turn_left(coords1 + Direction::South,
                                       Direction::West,
                                       grid)
                } else {
                    true
                }
            }
            // TODO: other cases
            (_, _, _) => true,
        };
        self.prev = self.curr;
        self.curr = Some(zone);
        more
    }

    pub fn finish(mut self, grid: &mut EditGrid) {
        match (self.changed, self.prev, self.curr) {
            (_, Some(Zone::East(coords1)), Some(Zone::Center(coords2))) => {
                if coords1 == coords2 {
                    self.try_split(coords1, Direction::East, grid);
                } else if coords1 + Direction::East == coords2 {
                    self.try_split(coords2, Direction::West, grid);
                }
            }
            (_, Some(Zone::South(coords1)), Some(Zone::Center(coords2))) => {
                if coords1 == coords2 {
                    self.try_split(coords1, Direction::South, grid);
                } else if coords1 + Direction::South == coords2 {
                    self.try_split(coords2, Direction::North, grid);
                }
            }
            (false, None, Some(Zone::Center(coords))) => {
                self.try_toggle_cross(coords, grid);
            }
            (false, None, Some(Zone::East(coords))) => {
                self.try_remove_stub(coords, Direction::East, grid);
            }
            (false, None, Some(Zone::South(coords))) => {
                self.try_remove_stub(coords, Direction::South, grid);
            }
            (_, _, _) => {}
        }
        grid.commit_provisional_changes();
    }

    fn try_start_stub(&mut self, coords: Coords, dir: Direction,
                      grid: &mut EditGrid)
                      -> bool {
        let changes = vec![GridChange::AddStubWire(coords, dir)];
        if grid.try_mutate_provisionally(changes) {
            self.changed = true;
        }
        true
    }

    fn try_remove_stub(&mut self, coords: Coords, dir: Direction,
                       grid: &mut EditGrid) {
        let changes = vec![GridChange::RemoveStubWire(coords, dir)];
        if grid.try_mutate_provisionally(changes) {
            self.changed = true;
        }
    }

    fn try_toggle_cross(&mut self, coords: Coords, grid: &mut EditGrid) {
        match (grid.wire_shape_at(coords, Direction::East),
                 grid.wire_shape_at(coords, Direction::South)) {
            (Some(WireShape::Straight), Some(WireShape::Straight)) |
            (Some(WireShape::Cross), _) => {
                let changes = vec![GridChange::ToggleCrossWire(coords)];
                if !grid.try_mutate_provisionally(changes) {
                    debug_log!("WARNING: try_toggle_cross mutation failed");
                }
                self.changed = true;
            }
            (_, _) => {}
        }
    }

    fn try_straight(&mut self, coords: Coords, dir: Direction,
                    grid: &mut EditGrid)
                    -> bool {
        let mut changes = Vec::<GridChange>::new();
        if grid.wire_shape_at(coords, dir).is_none() {
            changes.push(GridChange::AddStubWire(coords, dir));
        }
        if grid.wire_shape_at(coords, -dir).is_none() {
            changes.push(GridChange::AddStubWire(coords, -dir));
        }
        changes.push(GridChange::ToggleCenterWire(coords, dir, -dir));
        if grid.wire_shape_at(coords, dir) == Some(WireShape::Straight) &&
            grid.wire_shape_at(coords + dir, -dir) == Some(WireShape::Stub)
        {
            changes.push(GridChange::RemoveStubWire(coords, dir));
        }
        if grid.wire_shape_at(coords, -dir) == Some(WireShape::Straight) &&
            grid.wire_shape_at(coords - dir, dir) == Some(WireShape::Stub)
        {
            changes.push(GridChange::RemoveStubWire(coords, -dir));
        }
        let success = grid.try_mutate_provisionally(changes);
        self.changed |= success;
        success
    }

    fn try_turn_left(&mut self, coords: Coords, dir: Direction,
                     grid: &mut EditGrid)
                     -> bool {
        let dir2 = dir.rotate_cw();
        let mut changes = Vec::<GridChange>::new();
        if grid.wire_shape_at(coords, dir).is_none() {
            changes.push(GridChange::AddStubWire(coords, dir));
        }
        if grid.wire_shape_at(coords, dir2).is_none() {
            changes.push(GridChange::AddStubWire(coords, dir2));
        }
        changes.push(GridChange::ToggleCenterWire(coords, dir, dir2));
        if grid.wire_shape_at(coords, dir) == Some(WireShape::TurnLeft) &&
            grid.wire_shape_at(coords + dir, -dir) == Some(WireShape::Stub)
        {
            changes.push(GridChange::RemoveStubWire(coords, dir));
        }
        if grid.wire_shape_at(coords, dir2) == Some(WireShape::TurnRight) &&
            grid.wire_shape_at(coords + dir2, -dir2) == Some(WireShape::Stub)
        {
            changes.push(GridChange::RemoveStubWire(coords, dir2));
        }
        let success = grid.try_mutate_provisionally(changes);
        self.changed |= success;
        success
    }

    fn try_split(&mut self, coords: Coords, dir: Direction,
                 grid: &mut EditGrid)
                 -> bool {
        let mut changes = Vec::<GridChange>::new();
        let shape = grid.wire_shape_at(coords, dir);
        if shape.is_none() {
            changes.push(GridChange::AddStubWire(coords, dir));
        }
        changes.push(GridChange::ToggleSplitWire(coords, dir));
        if shape.is_some() && shape != Some(WireShape::Stub) &&
            grid.wire_shape_at(coords + dir, -dir) == Some(WireShape::Stub)
        {
            changes.push(GridChange::RemoveStubWire(coords, dir));
        }
        let success = grid.try_mutate_provisionally(changes);
        self.changed |= success;
        success
    }
}

//===========================================================================//
