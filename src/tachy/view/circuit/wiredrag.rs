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

use cgmath::{Point2, vec2};
use std::collections::HashMap;
use tachy::geom::{AsInt, Coords, Direction, Polygon};
use tachy::gui::Ui;
use tachy::save::WireShape;
use tachy::state::{EditGrid, GridChange};

//===========================================================================//

const ZONE_CENTER_SEMI_SIZE: f32 = 0.3;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Zone {
    Center(Coords),
    East(Coords),
    South(Coords),
}

impl Zone {
    fn from_grid_pt(grid_pt: Point2<f32>) -> Zone {
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

    fn along_line(start_grid_pt: Point2<f32>, end_grid_pt: Point2<f32>)
                  -> Vec<Zone> {
        let start_zone = Zone::from_grid_pt(start_grid_pt);
        let end_zone = Zone::from_grid_pt(end_grid_pt);
        let mut zones = vec![start_zone];
        let mut current_zone = start_zone;
        while current_zone != end_zone {
            let polygon = current_zone.polygon();
            let intersection =
                polygon.edge_intersection(end_grid_pt, start_grid_pt);
            let edge = match intersection {
                Some((edge, _)) => edge,
                None => {
                    // TODO: Figure out why this happens, and fix it.
                    debug_log!("WARNING: no intersection for zone={:?} \
                                polygon={:?} start={:?} end={:?} so_far={:?}",
                               current_zone,
                               polygon,
                               start_grid_pt,
                               end_grid_pt,
                               zones);
                    return zones;
                }
            };
            current_zone = match current_zone {
                Zone::Center(coords) => {
                    match edge {
                        0 => Zone::East(coords),
                        1 => Zone::South(coords),
                        2 => Zone::East(coords + Direction::West),
                        3 => Zone::South(coords + Direction::North),
                        _ => unreachable!(),
                    }
                }
                Zone::East(coords) => {
                    match edge {
                        0 => Zone::South(coords + vec2(1, -1)),
                        1 => Zone::Center(coords + Direction::East),
                        2 => Zone::South(coords + Direction::East),
                        3 => Zone::South(coords),
                        4 => Zone::Center(coords),
                        5 => Zone::South(coords + Direction::North),
                        _ => unreachable!(),
                    }
                }
                Zone::South(coords) => {
                    match edge {
                        0 => Zone::East(coords + Direction::West),
                        1 => Zone::Center(coords),
                        2 => Zone::East(coords),
                        3 => Zone::East(coords + Direction::South),
                        4 => Zone::Center(coords + Direction::South),
                        5 => Zone::East(coords + vec2(-1, 1)),
                        _ => unreachable!(),
                    }
                }
            };
            zones.push(current_zone);
        }
        zones
    }

    fn polygon(&self) -> Polygon {
        match *self {
            Zone::Center(coords) => {
                let cx = (coords.x as f32) + 0.5;
                let cy = (coords.y as f32) + 0.5;
                Polygon::new(vec![
                    Point2::new(
                        cx + ZONE_CENTER_SEMI_SIZE,
                        cy - ZONE_CENTER_SEMI_SIZE
                    ),
                    Point2::new(
                        cx + ZONE_CENTER_SEMI_SIZE,
                        cy + ZONE_CENTER_SEMI_SIZE
                    ),
                    Point2::new(
                        cx - ZONE_CENTER_SEMI_SIZE,
                        cy + ZONE_CENTER_SEMI_SIZE
                    ),
                    Point2::new(
                        cx - ZONE_CENTER_SEMI_SIZE,
                        cy - ZONE_CENTER_SEMI_SIZE
                    ),
                ])
            }
            Zone::East(coords) => {
                let cx = (coords.x as f32) + 1.0;
                let cy = (coords.y as f32) + 0.5;
                Polygon::new(vec![
                    Point2::new(cx, cy - 0.5),
                    Point2::new(
                        cx + 0.5 - ZONE_CENTER_SEMI_SIZE,
                        cy - ZONE_CENTER_SEMI_SIZE
                    ),
                    Point2::new(
                        cx + 0.5 - ZONE_CENTER_SEMI_SIZE,
                        cy + ZONE_CENTER_SEMI_SIZE
                    ),
                    Point2::new(cx, cy + 0.5),
                    Point2::new(
                        cx - 0.5 + ZONE_CENTER_SEMI_SIZE,
                        cy + ZONE_CENTER_SEMI_SIZE
                    ),
                    Point2::new(
                        cx - 0.5 + ZONE_CENTER_SEMI_SIZE,
                        cy - ZONE_CENTER_SEMI_SIZE
                    ),
                ])
            }
            Zone::South(coords) => {
                let cx = (coords.x as f32) + 0.5;
                let cy = (coords.y as f32) + 1.0;
                Polygon::new(vec![
                    Point2::new(cx - 0.5, cy),
                    Point2::new(
                        cx - ZONE_CENTER_SEMI_SIZE,
                        cy - 0.5 + ZONE_CENTER_SEMI_SIZE
                    ),
                    Point2::new(
                        cx + ZONE_CENTER_SEMI_SIZE,
                        cy - 0.5 + ZONE_CENTER_SEMI_SIZE
                    ),
                    Point2::new(cx + 0.5, cy),
                    Point2::new(
                        cx + ZONE_CENTER_SEMI_SIZE,
                        cy + 0.5 - ZONE_CENTER_SEMI_SIZE
                    ),
                    Point2::new(
                        cx - ZONE_CENTER_SEMI_SIZE,
                        cy + 0.5 - ZONE_CENTER_SEMI_SIZE
                    ),
                ])
            }
        }
    }
}

//===========================================================================//

pub struct WireDrag {
    last_pt: Option<Point2<f32>>,
    curr: Option<Zone>,
    prev: Option<Zone>,
    changed: bool,
}

// TODO: enforce wires must be in bounds
// TODO: enforce wires can't be created under chips
impl WireDrag {
    pub fn new() -> WireDrag {
        WireDrag {
            last_pt: None,
            curr: None,
            prev: None,
            changed: false,
        }
    }

    pub fn move_to(&mut self, grid_pt: Point2<f32>, ui: &mut Ui,
                   grid: &mut EditGrid)
                   -> bool {
        ui.request_redraw(); // TODO: only if changes were made
        let last_pt = self.last_pt;
        self.last_pt = Some(grid_pt);
        if let Some(start) = last_pt {
            for zone in Zone::along_line(start, grid_pt) {
                if !self.move_to_zone(zone, grid) {
                    return false;
                }
            }
            return true;
        } else {
            return self.move_to_zone(Zone::from_grid_pt(grid_pt), grid);
        }
    }

    fn move_to_zone(&mut self, zone: Zone, grid: &mut EditGrid) -> bool {
        if self.curr == Some(zone) {
            return true;
        }
        let more = match (self.prev, self.curr, zone) {
            (_, None, Zone::Center(_)) => true,
            (_, None, Zone::East(coords)) => {
                self.try_start_stub(coords, Direction::East, grid)
            }
            (_, None, Zone::South(coords)) => {
                self.try_start_stub(coords, Direction::South, grid)
            }
            (_, Some(Zone::Center(_)), Zone::Center(_)) => {
                debug_log!("WARNING: Pattern (_, Center, Center) shouldn't \
                            happen!");
                false
            }
            (_, Some(Zone::East(_)), Zone::East(_)) => {
                debug_log!("WARNING: Pattern (_, East, East) shouldn't \
                            happen!");
                false
            }
            (_, Some(Zone::South(_)), Zone::South(_)) => {
                debug_log!("WARNING: Pattern (_, East, East) shouldn't \
                            happen!");
                false
            }
            (_, Some(Zone::East(_)), Zone::Center(_)) => true,
            (_, Some(Zone::South(_)), Zone::Center(_)) => true,
            (None, Some(Zone::Center(coords1)), Zone::East(coords2)) => {
                if coords1 == coords2 {
                    self.try_split(coords1, Direction::East, grid)
                } else if coords1 + Direction::West == coords2 {
                    self.try_split(coords1, Direction::West, grid)
                } else {
                    debug_log!("Pattern (None, Center, East) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    true
                }
            }
            (None, Some(Zone::Center(coords1)), Zone::South(coords2)) => {
                if coords1 == coords2 {
                    self.try_split(coords1, Direction::South, grid)
                } else if coords1 + Direction::North == coords2 {
                    self.try_split(coords1, Direction::North, grid)
                } else {
                    debug_log!("Pattern (None, Center, South) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    true
                }
            }
            (Some(Zone::Center(_)), Some(Zone::Center(_)), Zone::East(_)) => {
                debug_log!("WARNING: Pattern (Center, Center, East) \
                            shouldn't happen!");
                false
            }
            (Some(Zone::Center(_)), Some(Zone::Center(_)), Zone::South(_)) => {
                debug_log!("WARNING: Pattern (Center, Center, South) \
                            shouldn't happen!");
                false
            }
            (Some(Zone::East(coords1)),
             Some(Zone::Center(_)),
             Zone::East(coords2)) => {
                if coords1 + Direction::East == coords2 {
                    self.try_straight(coords2, Direction::East, grid)
                } else if coords1 + Direction::West == coords2 {
                    self.try_straight(coords1, Direction::West, grid)
                } else {
                    debug_log!("Pattern (East, Center, East) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    true
                }
            }
            (Some(Zone::South(coords1)),
             Some(Zone::Center(_)),
             Zone::South(coords2)) => {
                if coords1 + Direction::South == coords2 {
                    self.try_straight(coords2, Direction::South, grid)
                } else if coords1 + Direction::North == coords2 {
                    self.try_straight(coords1, Direction::North, grid)
                } else {
                    debug_log!("Pattern (South, Center, South) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    true
                }
            }
            (Some(Zone::East(c1)),
             Some(Zone::Center(c2)),
             Zone::South(c3)) => {
                if c1 == c2 && c2 == c3 {
                    self.try_turn_left(c2, Direction::East, grid)
                } else if c1 == c2 && c2 + Direction::North == c3 {
                    self.try_turn_left(c2, Direction::North, grid)
                } else if c1 + Direction::East == c2 && c2 == c3 {
                    self.try_turn_left(c2, Direction::South, grid)
                } else if c1 + Direction::East == c2 &&
                           c2 + Direction::North == c3
                {
                    self.try_turn_left(c2, Direction::West, grid)
                } else {
                    debug_log!("Pattern (East, Center, South) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    true
                }
            }
            (Some(Zone::South(c1)),
             Some(Zone::Center(c2)),
             Zone::East(c3)) => {
                if c1 == c2 && c2 == c3 {
                    self.try_turn_left(c2, Direction::East, grid)
                } else if c1 == c2 && c2 + Direction::West == c3 {
                    self.try_turn_left(c2, Direction::South, grid)
                } else if c1 + Direction::South == c2 && c2 == c3 {
                    self.try_turn_left(c2, Direction::North, grid)
                } else if c1 + Direction::South == c2 &&
                           c2 + Direction::West == c3
                {
                    self.try_turn_left(c2, Direction::West, grid)
                } else {
                    debug_log!("Pattern (South, Center, East) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
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
                    debug_log!("Pattern (_, East, South) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
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
                    debug_log!("Pattern (_, South, East) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    true
                }
            }
        };
        self.prev = self.curr;
        self.curr = Some(zone);
        more
    }

    pub fn finish(mut self, ui: &mut Ui, grid: &mut EditGrid) {
        ui.request_redraw(); // TODO: only if changes were made
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
        let changes = vec![GridChange::add_stub_wire(coords, dir)];
        if grid.try_mutate_provisionally(changes) {
            self.changed = true;
        }
        true
    }

    fn try_remove_stub(&mut self, coords: Coords, dir: Direction,
                       grid: &mut EditGrid) {
        let changes = vec![GridChange::remove_stub_wire(coords, dir)];
        if grid.try_mutate_provisionally(changes) {
            self.changed = true;
        }
    }

    fn try_toggle_cross(&mut self, coords: Coords, grid: &mut EditGrid) {
        let changes = vec![GridChange::ToggleCrossWire(coords)];
        if grid.try_mutate_provisionally(changes) {
            self.changed = true;
        }
    }

    fn try_straight(&mut self, coords: Coords, dir: Direction,
                    grid: &mut EditGrid)
                    -> bool {
        let mut old_wires = HashMap::<(Coords, Direction), WireShape>::new();
        let mut new_wires = HashMap::<(Coords, Direction), WireShape>::new();
        for &dir in &[dir, -dir] {
            match grid.wire_shape_at(coords, dir) {
                None => {
                    new_wires.insert((coords, dir), WireShape::Straight);
                    new_wires.insert((coords + dir, -dir), WireShape::Stub);
                }
                Some(WireShape::Stub) => {
                    old_wires.insert((coords, dir), WireShape::Stub);
                    new_wires.insert((coords, dir), WireShape::Straight);
                }
                Some(WireShape::Straight) => {
                    old_wires.insert((coords, dir), WireShape::Straight);
                    if grid.wire_shape_at(coords + dir, -dir) ==
                        Some(WireShape::Stub)
                    {
                        old_wires
                            .insert((coords + dir, -dir), WireShape::Stub);
                    } else {
                        new_wires.insert((coords, dir), WireShape::Stub);
                    }
                }
                _ => return false,
            }
        }
        let changes = vec![GridChange::ReplaceWires(old_wires, new_wires)];
        let success = grid.try_mutate_provisionally(changes);
        if !success {
            debug_log!("try_straight failed: coords={:?}, dir={:?}",
                       coords,
                       dir);
        }
        self.changed |= success;
        success
    }

    fn try_turn_left(&mut self, coords: Coords, dir: Direction,
                     grid: &mut EditGrid)
                     -> bool {
        let mut old_wires = HashMap::<(Coords, Direction), WireShape>::new();
        let mut new_wires = HashMap::<(Coords, Direction), WireShape>::new();
        for &(dir, turn) in &[
            (dir, WireShape::TurnLeft),
            (dir.rotate_cw(), WireShape::TurnRight),
        ]
        {
            match grid.wire_shape_at(coords, dir) {
                None => {
                    new_wires.insert((coords, dir), turn);
                    new_wires.insert((coords + dir, -dir), WireShape::Stub);
                }
                Some(WireShape::Stub) => {
                    old_wires.insert((coords, dir), WireShape::Stub);
                    new_wires.insert((coords, dir), turn);
                }
                Some(shape) if shape == turn => {
                    old_wires.insert((coords, dir), turn);
                    if grid.wire_shape_at(coords + dir, -dir) ==
                        Some(WireShape::Stub)
                    {
                        old_wires
                            .insert((coords + dir, -dir), WireShape::Stub);
                    } else {
                        new_wires.insert((coords, dir), WireShape::Stub);
                    }
                }
                _ => return false,
            }
        }
        let changes = vec![GridChange::ReplaceWires(old_wires, new_wires)];
        let success = grid.try_mutate_provisionally(changes);
        if !success {
            debug_log!("try_turn_left failed: coords={:?}, dir={:?}",
                       coords,
                       dir);
        }
        self.changed |= success;
        success
    }

    fn try_split(&mut self, coords: Coords, dir: Direction,
                 grid: &mut EditGrid)
                 -> bool {
        let mut changes = Vec::<GridChange>::new();
        let shape = grid.wire_shape_at(coords, dir);
        if shape.is_none() {
            changes.push(GridChange::add_stub_wire(coords, dir));
        }
        changes.push(GridChange::ToggleSplitWire(coords, dir));
        if shape.is_some() && shape != Some(WireShape::Stub) &&
            grid.wire_shape_at(coords + dir, -dir) == Some(WireShape::Stub)
        {
            changes.push(GridChange::remove_stub_wire(coords, dir));
        }
        let success = grid.try_mutate_provisionally(changes);
        if !success {
            debug_log!("try_split failed: coords={:?}, dir={:?}", coords, dir);
        }
        self.changed |= success;
        success
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::Zone;
    use cgmath::Point2;
    use tachy::geom::Coords;

    #[test]
    fn zones_along_line() {
        assert_eq!(Zone::along_line(Point2::new(2.4, 6.5),
                                    Point2::new(2.6, 6.5)),
                   vec![Zone::Center(Coords::new(2, 6))]);
        assert_eq!(
            Zone::along_line(Point2::new(2.4, 6.5), Point2::new(3.6, 6.5)),
            vec![
                Zone::Center(Coords::new(2, 6)),
                Zone::East(Coords::new(2, 6)),
                Zone::Center(Coords::new(3, 6)),
            ]
        );
        assert_eq!(
            Zone::along_line(Point2::new(2.4, 6.5), Point2::new(2.6, 5.5)),
            vec![
                Zone::Center(Coords::new(2, 6)),
                Zone::South(Coords::new(2, 5)),
                Zone::Center(Coords::new(2, 5)),
            ]
        );
        assert_eq!(
            Zone::along_line(Point2::new(2.9, 5.5), Point2::new(2.9, 6.5)),
            vec![
                Zone::East(Coords::new(2, 5)),
                Zone::South(Coords::new(2, 5)),
                Zone::East(Coords::new(2, 6)),
            ]
        );
    }
}

//===========================================================================//
