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
use tachy::gui::{Sound, Ui};
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
                    debug_warn!("no intersection for zone={:?} \
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

#[derive(Clone, Copy, Eq, PartialEq)]
#[must_use = "must not ignore DragResult"]
enum DragResult {
    Changed,
    Unchanged,
    Stop,
}

//===========================================================================//

pub struct WireDrag {
    last_pt: Option<Point2<f32>>,
    curr: Option<Zone>,
    prev: Option<Zone>,
    changed: bool,
}

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
        let last_pt = self.last_pt;
        self.last_pt = Some(grid_pt);
        let mut drag_result = DragResult::Unchanged;
        if let Some(start) = last_pt {
            for zone in Zone::along_line(start, grid_pt) {
                drag_result = self.move_to_zone(zone, grid);
                if drag_result == DragResult::Stop {
                    break;
                }
            }
        } else {
            drag_result = self.move_to_zone(Zone::from_grid_pt(grid_pt), grid);
        }
        if drag_result == DragResult::Changed {
            self.changed = true;
            ui.audio().play_sound(Sound::DragWire);
            ui.request_redraw();
        }
        drag_result != DragResult::Stop
    }

    fn move_to_zone(&mut self, zone: Zone, grid: &mut EditGrid) -> DragResult {
        if self.curr == Some(zone) {
            return DragResult::Unchanged;
        }
        let drag_result = match (self.prev, self.curr, zone) {
            (_, None, Zone::Center(_)) => DragResult::Unchanged,
            (_, None, Zone::East(coords)) => {
                WireDrag::start_stub(coords, Direction::East, grid)
            }
            (_, None, Zone::South(coords)) => {
                WireDrag::start_stub(coords, Direction::South, grid)
            }
            (_, Some(Zone::Center(_)), Zone::Center(_)) => {
                debug_warn!("Pattern (_, Center, Center) shouldn't happen!");
                DragResult::Stop
            }
            (_, Some(Zone::East(_)), Zone::East(_)) => {
                debug_warn!("Pattern (_, East, East) shouldn't happen!");
                DragResult::Stop
            }
            (_, Some(Zone::South(_)), Zone::South(_)) => {
                debug_warn!("Pattern (_, South, South) shouldn't happen!");
                DragResult::Stop
            }
            (_, Some(Zone::East(_)), Zone::Center(_)) => DragResult::Unchanged,
            (_, Some(Zone::South(_)), Zone::Center(_)) => {
                DragResult::Unchanged
            }
            (None, Some(Zone::Center(coords1)), Zone::East(coords2)) => {
                if coords1 == coords2 {
                    WireDrag::split(coords1, Direction::East, grid)
                } else if coords1 + Direction::West == coords2 {
                    WireDrag::split(coords1, Direction::West, grid)
                } else {
                    debug_log!("Pattern (None, Center, East) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    DragResult::Unchanged
                }
            }
            (None, Some(Zone::Center(coords1)), Zone::South(coords2)) => {
                if coords1 == coords2 {
                    WireDrag::split(coords1, Direction::South, grid)
                } else if coords1 + Direction::North == coords2 {
                    WireDrag::split(coords1, Direction::North, grid)
                } else {
                    debug_log!("Pattern (None, Center, South) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    DragResult::Unchanged
                }
            }
            (Some(Zone::Center(_)), Some(Zone::Center(_)), Zone::East(_)) => {
                debug_warn!("Pattern (Center, Center, East) shouldn't \
                             happen!");
                DragResult::Stop
            }
            (Some(Zone::Center(_)), Some(Zone::Center(_)), Zone::South(_)) => {
                debug_warn!("Pattern (Center, Center, South) shouldn't \
                             happen!");
                DragResult::Stop
            }
            (Some(Zone::East(coords1)),
             Some(Zone::Center(_)),
             Zone::East(coords2)) => {
                if coords1 + Direction::East == coords2 {
                    WireDrag::straight(coords2, Direction::West, grid)
                } else if coords1 + Direction::West == coords2 {
                    WireDrag::straight(coords1, Direction::East, grid)
                } else {
                    debug_log!("Pattern (East, Center, East) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    DragResult::Unchanged
                }
            }
            (Some(Zone::South(coords1)),
             Some(Zone::Center(_)),
             Zone::South(coords2)) => {
                if coords1 + Direction::South == coords2 {
                    WireDrag::straight(coords2, Direction::North, grid)
                } else if coords1 + Direction::North == coords2 {
                    WireDrag::straight(coords1, Direction::South, grid)
                } else {
                    debug_log!("Pattern (South, Center, South) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    DragResult::Unchanged
                }
            }
            (Some(Zone::East(c1)),
             Some(Zone::Center(c2)),
             Zone::South(c3)) => {
                if c1 == c2 && c2 == c3 {
                    WireDrag::turn_left(c2, Direction::East, grid)
                } else if c1 == c2 && c2 + Direction::North == c3 {
                    WireDrag::turn_left(c2, Direction::North, grid)
                } else if c1 + Direction::East == c2 && c2 == c3 {
                    WireDrag::turn_left(c2, Direction::South, grid)
                } else if c1 + Direction::East == c2 &&
                           c2 + Direction::North == c3
                {
                    WireDrag::turn_left(c2, Direction::West, grid)
                } else {
                    debug_log!("Pattern (East, Center, South) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    DragResult::Unchanged
                }
            }
            (Some(Zone::South(c1)),
             Some(Zone::Center(c2)),
             Zone::East(c3)) => {
                if c1 == c2 && c2 == c3 {
                    WireDrag::turn_left(c2, Direction::East, grid)
                } else if c1 == c2 && c2 + Direction::West == c3 {
                    WireDrag::turn_left(c2, Direction::South, grid)
                } else if c1 + Direction::South == c2 && c2 == c3 {
                    WireDrag::turn_left(c2, Direction::North, grid)
                } else if c1 + Direction::South == c2 &&
                           c2 + Direction::West == c3
                {
                    WireDrag::turn_left(c2, Direction::West, grid)
                } else {
                    debug_log!("Pattern (South, Center, East) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    DragResult::Unchanged
                }
            }
            (_, Some(Zone::East(coords1)), Zone::South(coords2)) => {
                if coords1 == coords2 {
                    WireDrag::turn_left(coords1, Direction::East, grid)
                } else if coords1 + Direction::North == coords2 {
                    WireDrag::turn_left(coords1, Direction::North, grid)
                } else if coords1 + Direction::East == coords2 {
                    WireDrag::turn_left(coords2, Direction::South, grid)
                } else if coords1 + Direction::East ==
                           coords2 + Direction::South
                {
                    WireDrag::turn_left(coords1 + Direction::East,
                                        Direction::West,
                                        grid)
                } else {
                    debug_log!("Pattern (_, East, South) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    DragResult::Unchanged
                }
            }
            (_, Some(Zone::South(coords1)), Zone::East(coords2)) => {
                if coords1 == coords2 {
                    WireDrag::turn_left(coords1, Direction::East, grid)
                } else if coords1 + Direction::South == coords2 {
                    WireDrag::turn_left(coords2, Direction::North, grid)
                } else if coords1 + Direction::West == coords2 {
                    WireDrag::turn_left(coords1, Direction::South, grid)
                } else if coords1 + Direction::South ==
                           coords2 + Direction::East
                {
                    WireDrag::turn_left(coords1 + Direction::South,
                                        Direction::West,
                                        grid)
                } else {
                    debug_log!("Pattern (_, South, East) does not match \
                                {:?}, {:?}, {:?}",
                               self.prev,
                               self.curr,
                               zone);
                    DragResult::Unchanged
                }
            }
        };
        self.prev = self.curr;
        self.curr = Some(zone);
        drag_result
    }

    pub fn finish(self, ui: &mut Ui, grid: &mut EditGrid) {
        let drag_result = match (self.changed, self.prev, self.curr) {
            (_, Some(Zone::East(coords1)), Some(Zone::Center(coords2))) => {
                if coords1 == coords2 {
                    WireDrag::split(coords1, Direction::East, grid)
                } else if coords1 + Direction::East == coords2 {
                    WireDrag::split(coords2, Direction::West, grid)
                } else {
                    DragResult::Unchanged
                }
            }
            (_, Some(Zone::South(coords1)), Some(Zone::Center(coords2))) => {
                if coords1 == coords2 {
                    WireDrag::split(coords1, Direction::South, grid)
                } else if coords1 + Direction::South == coords2 {
                    WireDrag::split(coords2, Direction::North, grid)
                } else {
                    DragResult::Unchanged
                }
            }
            (false, None, Some(Zone::Center(coords))) => {
                WireDrag::toggle_cross(coords, grid)
            }
            (false, None, Some(Zone::East(coords))) => {
                WireDrag::remove_stub(coords, Direction::East, grid)
            }
            (false, None, Some(Zone::South(coords))) => {
                WireDrag::remove_stub(coords, Direction::South, grid)
            }
            (_, _, _) => DragResult::Unchanged,
        };
        if drag_result == DragResult::Changed {
            ui.audio().play_sound(Sound::DragWire);
            ui.request_redraw();
        }
        grid.commit_provisional_changes();
    }

    pub fn try_toggle_cross(coords: Coords, grid: &mut EditGrid) -> bool {
        WireDrag::toggle_cross(coords, grid) == DragResult::Changed
    }

    fn toggle_cross(coords: Coords, grid: &mut EditGrid) -> DragResult {
        if !grid.bounds().contains_point(coords) || grid.has_chip_at(coords) {
            return DragResult::Stop;
        }
        let (old_shape, new_shape) =
            if grid.wire_shape_at(coords, Direction::East) ==
                Some(WireShape::Cross)
            {
                (WireShape::Cross, WireShape::Straight)
            } else {
                (WireShape::Straight, WireShape::Cross)
            };
        let mut old = HashMap::<(Coords, Direction), WireShape>::new();
        let mut new = HashMap::<(Coords, Direction), WireShape>::new();
        for dir in Direction::all() {
            old.insert((coords, dir), old_shape);
            new.insert((coords, dir), new_shape);
        }
        let changes = vec![GridChange::ReplaceWires(old, new)];
        if grid.try_mutate_provisionally(changes) {
            return DragResult::Changed;
        } else {
            return DragResult::Unchanged;
        }
    }

    fn start_stub(coords: Coords, dir: Direction, grid: &mut EditGrid)
                  -> DragResult {
        let changes = vec![GridChange::add_stub_wire(coords, dir)];
        if grid.try_mutate_provisionally(changes) {
            DragResult::Changed
        } else {
            DragResult::Unchanged
        }
    }

    fn remove_stub(coords: Coords, dir: Direction, grid: &mut EditGrid)
                   -> DragResult {
        let changes = vec![GridChange::remove_stub_wire(coords, dir)];
        if grid.try_mutate_provisionally(changes) {
            DragResult::Changed
        } else {
            DragResult::Unchanged
        }
    }

    fn straight(coords: Coords, dir: Direction, grid: &mut EditGrid)
                -> DragResult {
        if !grid.bounds().contains_point(coords) || grid.has_chip_at(coords) {
            return DragResult::Stop;
        }
        let mut old = HashMap::<(Coords, Direction), WireShape>::new();
        let mut new = HashMap::<(Coords, Direction), WireShape>::new();
        match (grid.wire_shape_at(coords, dir),
                 grid.wire_shape_at(coords, -dir)) {
            (None, None) |
            (Some(WireShape::Stub), None) |
            (None, Some(WireShape::Stub)) |
            (Some(WireShape::Stub), Some(WireShape::Stub)) => {
                new.insert((coords, dir), WireShape::Straight);
                new.insert((coords, -dir), WireShape::Straight);
                stub(coords + dir, -dir, grid, &mut old, &mut new);
                stub(coords - dir, dir, grid, &mut old, &mut new);
            }
            (Some(WireShape::Straight), Some(WireShape::Straight)) => {
                old.insert((coords, dir), WireShape::Straight);
                old.insert((coords, -dir), WireShape::Straight);
                stub(coords, dir, grid, &mut old, &mut new);
                stub(coords, -dir, grid, &mut old, &mut new);
            }
            (Some(WireShape::TurnLeft), None) |
            (Some(WireShape::TurnLeft), Some(WireShape::Stub)) => {
                let side = dir.rotate_cw();
                old.insert((coords, dir), WireShape::TurnLeft);
                old.insert((coords, side), WireShape::TurnRight);
                new.insert((coords, side), WireShape::TurnLeft);
                new.insert((coords, -dir), WireShape::TurnRight);
                stub(coords, dir, grid, &mut old, &mut new);
                stub(coords - dir, dir, grid, &mut old, &mut new);
            }
            (Some(WireShape::TurnRight), None) |
            (Some(WireShape::TurnRight), Some(WireShape::Stub)) => {
                let side = dir.rotate_ccw();
                old.insert((coords, dir), WireShape::TurnRight);
                old.insert((coords, side), WireShape::TurnLeft);
                new.insert((coords, side), WireShape::TurnRight);
                new.insert((coords, -dir), WireShape::TurnLeft);
                stub(coords, dir, grid, &mut old, &mut new);
                stub(coords - dir, dir, grid, &mut old, &mut new);
            }
            (_, _) => return DragResult::Stop,
        }
        let changes = vec![GridChange::ReplaceWires(old, new)];
        if grid.try_mutate_provisionally(changes) {
            return DragResult::Changed;
        } else {
            debug_warn!("WireDrag::straight failed: coords={:?}, dir={:?}",
                        coords,
                        dir);
            return DragResult::Stop;
        }
    }

    fn turn_left(coords: Coords, dir: Direction, grid: &mut EditGrid)
                 -> DragResult {
        if !grid.bounds().contains_point(coords) || grid.has_chip_at(coords) {
            return DragResult::Stop;
        }
        let mut old = HashMap::<(Coords, Direction), WireShape>::new();
        let mut new = HashMap::<(Coords, Direction), WireShape>::new();
        let side = dir.rotate_cw();
        match (grid.wire_shape_at(coords, dir),
                 grid.wire_shape_at(coords, side)) {
            (None, None) |
            (Some(WireShape::Stub), None) |
            (None, Some(WireShape::Stub)) |
            (Some(WireShape::Stub), Some(WireShape::Stub)) => {
                new.insert((coords, dir), WireShape::TurnLeft);
                new.insert((coords, side), WireShape::TurnRight);
                stub(coords + dir, -dir, grid, &mut old, &mut new);
                stub(coords + side, -side, grid, &mut old, &mut new);
            }
            (Some(WireShape::TurnLeft), Some(WireShape::TurnRight)) => {
                old.insert((coords, dir), WireShape::TurnLeft);
                old.insert((coords, side), WireShape::TurnRight);
                stub(coords, dir, grid, &mut old, &mut new);
                stub(coords, side, grid, &mut old, &mut new);
            }
            (Some(WireShape::Straight), None) |
            (Some(WireShape::Straight), Some(WireShape::Stub)) => {
                old.insert((coords, dir), WireShape::Straight);
                old.insert((coords, -dir), WireShape::Straight);
                new.insert((coords, -dir), WireShape::TurnRight);
                new.insert((coords, side), WireShape::TurnLeft);
                stub(coords, dir, grid, &mut old, &mut new);
                stub(coords + side, -side, grid, &mut old, &mut new);
            }
            (None, Some(WireShape::Straight)) |
            (Some(WireShape::Stub), Some(WireShape::Straight)) => {
                old.insert((coords, side), WireShape::Straight);
                old.insert((coords, -side), WireShape::Straight);
                new.insert((coords, dir), WireShape::TurnRight);
                new.insert((coords, -side), WireShape::TurnLeft);
                stub(coords, side, grid, &mut old, &mut new);
                stub(coords + dir, -dir, grid, &mut old, &mut new);
            }
            (Some(WireShape::TurnRight), None) |
            (Some(WireShape::TurnRight), Some(WireShape::Stub)) => {
                old.insert((coords, dir), WireShape::TurnRight);
                old.insert((coords, -side), WireShape::TurnLeft);
                new.insert((coords, side), WireShape::Straight);
                new.insert((coords, -side), WireShape::Straight);
                stub(coords, dir, grid, &mut old, &mut new);
                stub(coords + side, -side, grid, &mut old, &mut new);
            }
            (None, Some(WireShape::TurnLeft)) |
            (Some(WireShape::Stub), Some(WireShape::TurnLeft)) => {
                old.insert((coords, -dir), WireShape::TurnRight);
                old.insert((coords, side), WireShape::TurnLeft);
                new.insert((coords, dir), WireShape::Straight);
                new.insert((coords, -dir), WireShape::Straight);
                stub(coords, side, grid, &mut old, &mut new);
                stub(coords + dir, -dir, grid, &mut old, &mut new);
            }
            (_, _) => return DragResult::Stop,
        }
        let changes = vec![GridChange::ReplaceWires(old, new)];
        if grid.try_mutate_provisionally(changes) {
            return DragResult::Changed;
        } else {
            debug_warn!("WireDrag::turn_left failed: coords={:?}, dir={:?}",
                        coords,
                        dir);
            return DragResult::Stop;
        }
    }

    fn split(coords: Coords, dir: Direction, grid: &mut EditGrid)
             -> DragResult {
        if !grid.bounds().contains_point(coords) || grid.has_chip_at(coords) {
            return DragResult::Stop;
        }
        let mut old = HashMap::<(Coords, Direction), WireShape>::new();
        let mut new = HashMap::<(Coords, Direction), WireShape>::new();
        let side = dir.rotate_cw();
        match (grid.wire_shape_at(coords, dir),
                 grid.wire_shape_at(coords, -dir),
                 grid.wire_shape_at(coords, side)) {
            (None, Some(WireShape::SplitTee), _) |
            (Some(WireShape::Stub), Some(WireShape::SplitTee), _) => {
                old.insert((coords, -dir), WireShape::SplitTee);
                old.insert((coords, side), WireShape::SplitLeft);
                old.insert((coords, -side), WireShape::SplitRight);
                for dir2 in Direction::all() {
                    new.insert((coords, dir2), WireShape::Cross);
                }
                stub(coords + dir, -dir, grid, &mut old, &mut new);
            }
            (Some(WireShape::Cross), _, _) => {
                for dir2 in Direction::all() {
                    old.insert((coords, dir2), WireShape::Cross);
                }
                new.insert((coords, -dir), WireShape::SplitTee);
                new.insert((coords, side), WireShape::SplitLeft);
                new.insert((coords, -side), WireShape::SplitRight);
                stub(coords, dir, grid, &mut old, &mut new);
            }
            (None, Some(WireShape::TurnLeft), _) |
            (Some(WireShape::Stub), Some(WireShape::TurnLeft), _) => {
                old.insert((coords, -dir), WireShape::TurnLeft);
                old.insert((coords, -side), WireShape::TurnRight);
                new.insert((coords, dir), WireShape::SplitRight);
                new.insert((coords, -dir), WireShape::SplitLeft);
                new.insert((coords, -side), WireShape::SplitTee);
                stub(coords + dir, -dir, grid, &mut old, &mut new);
            }
            (Some(WireShape::SplitRight), _, _) => {
                old.insert((coords, dir), WireShape::SplitRight);
                old.insert((coords, -dir), WireShape::SplitLeft);
                old.insert((coords, -side), WireShape::SplitTee);
                new.insert((coords, -dir), WireShape::TurnLeft);
                new.insert((coords, -side), WireShape::TurnRight);
                stub(coords, dir, grid, &mut old, &mut new);
            }
            (None, Some(WireShape::TurnRight), _) |
            (Some(WireShape::Stub), Some(WireShape::TurnRight), _) => {
                old.insert((coords, -dir), WireShape::TurnRight);
                old.insert((coords, side), WireShape::TurnLeft);
                new.insert((coords, dir), WireShape::SplitLeft);
                new.insert((coords, -dir), WireShape::SplitRight);
                new.insert((coords, side), WireShape::SplitTee);
                stub(coords + dir, -dir, grid, &mut old, &mut new);
            }
            (Some(WireShape::SplitLeft), _, _) => {
                old.insert((coords, dir), WireShape::SplitLeft);
                old.insert((coords, -dir), WireShape::SplitRight);
                old.insert((coords, side), WireShape::SplitTee);
                new.insert((coords, -dir), WireShape::TurnRight);
                new.insert((coords, side), WireShape::TurnLeft);
                stub(coords, dir, grid, &mut old, &mut new);
            }
            (None, _, Some(WireShape::Straight)) |
            (Some(WireShape::Stub), _, Some(WireShape::Straight)) => {
                old.insert((coords, side), WireShape::Straight);
                old.insert((coords, -side), WireShape::Straight);
                new.insert((coords, dir), WireShape::SplitTee);
                new.insert((coords, side), WireShape::SplitRight);
                new.insert((coords, -side), WireShape::SplitLeft);
                stub(coords + dir, -dir, grid, &mut old, &mut new);
            }
            (Some(WireShape::SplitTee), _, _) => {
                old.insert((coords, dir), WireShape::SplitTee);
                old.insert((coords, side), WireShape::SplitRight);
                old.insert((coords, -side), WireShape::SplitLeft);
                new.insert((coords, side), WireShape::Straight);
                new.insert((coords, -side), WireShape::Straight);
                stub(coords, dir, grid, &mut old, &mut new);
            }
            (_, _, _) => return DragResult::Stop,
        }
        let changes = vec![GridChange::ReplaceWires(old, new)];
        if grid.try_mutate_provisionally(changes) {
            return DragResult::Changed;
        } else {
            debug_warn!("WireDrag::split failed: coords={:?}, dir={:?}",
                        coords,
                        dir);
            return DragResult::Stop;
        }
    }
}

fn stub(coords: Coords, dir: Direction, grid: &EditGrid,
        old_wires: &mut HashMap<(Coords, Direction), WireShape>,
        new_wires: &mut HashMap<(Coords, Direction), WireShape>) {
    if grid.wire_shape_at(coords + dir, -dir) == Some(WireShape::Stub) {
        old_wires.insert((coords + dir, -dir), WireShape::Stub);
    } else {
        new_wires.insert((coords, dir), WireShape::Stub);
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
