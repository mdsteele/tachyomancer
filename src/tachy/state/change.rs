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

use std::collections::HashMap;
use tachy::geom::{Coords, CoordsRect, Direction, Orientation};
use tachy::save::{ChipType, WireShape};

//===========================================================================//

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GridChange {
    /// Removes the first set of wire fragments and adds the second set of wire
    /// fragments.
    ReplaceWires(HashMap<(Coords, Direction), WireShape>,
                 HashMap<(Coords, Direction), WireShape>),
    /// Places a chip onto the board.
    AddChip(Coords, ChipType, Orientation),
    /// Removes a chip from the board.
    RemoveChip(Coords, ChipType, Orientation),
    /// Change the bounds rect from the first rect to the second.
    SetBounds(CoordsRect, CoordsRect),
}

impl GridChange {
    pub(super) fn invert_and_collapse_group(mut changes: Vec<GridChange>)
                                            -> Vec<GridChange> {
        let mut new_changes = Vec::<GridChange>::new();
        while let Some(change) = changes.pop() {
            match (new_changes.pop(), change.invert()) {
                (Some(GridChange::ReplaceWires(mut old1, mut new1)),
                 GridChange::ReplaceWires(old2, new2)) => {
                    for (loc, shape) in old2.into_iter() {
                        if new1.get(&loc) == Some(&shape) {
                            new1.remove(&loc);
                        } else {
                            old1.insert(loc, shape);
                        }
                    }
                    for (loc, shape) in new2.into_iter() {
                        if old1.get(&loc) == Some(&shape) {
                            old1.remove(&loc);
                        } else {
                            new1.insert(loc, shape);
                        }
                    }
                    if !(old1.is_empty() && new1.is_empty()) {
                        new_changes.push(GridChange::ReplaceWires(old1, new1));
                    }
                }
                (Some(GridChange::AddChip(c1, t1, o1)),
                 GridChange::RemoveChip(c2, t2, o2))
                    if c1 == c2 && t1 == t2 && o1 == o2 => {}
                (Some(GridChange::RemoveChip(c1, t1, o1)),
                 GridChange::AddChip(c2, t2, o2))
                    if c1 == c2 && t1 == t2 && o1 == o2 => {}
                (Some(GridChange::SetBounds(r1, r2)),
                 GridChange::SetBounds(r3, r4)) => {
                    debug_assert_eq!(r2, r3);
                    if r1 != r4 {
                        new_changes.push(GridChange::SetBounds(r1, r4));
                    }
                }
                (opt_change1, change2) => {
                    if let Some(change1) = opt_change1 {
                        new_changes.push(change1);
                    }
                    match change2 {
                        GridChange::ReplaceWires(mut old, mut new) => {
                            new.retain(
                                |loc, shape| if old.get(loc) == Some(shape) {
                                    old.remove(loc);
                                    false
                                } else {
                                    true
                                },
                            );
                            if !(old.is_empty() && new.is_empty()) {
                                new_changes
                                    .push(GridChange::ReplaceWires(old, new));
                            }
                        }
                        GridChange::SetBounds(r1, r2) if r1 == r2 => {}
                        _ => new_changes.push(change2),
                    }
                }
            }
        }
        new_changes
    }

    pub(super) fn invert_group(changes: Vec<GridChange>) -> Vec<GridChange> {
        changes.into_iter().rev().map(GridChange::invert).collect()
    }

    fn invert(self) -> GridChange {
        match self {
            GridChange::ReplaceWires(old, new) => {
                GridChange::ReplaceWires(new, old)
            }
            GridChange::AddChip(c, t, o) => GridChange::RemoveChip(c, t, o),
            GridChange::RemoveChip(c, t, o) => GridChange::AddChip(c, t, o),
            GridChange::SetBounds(old, new) => GridChange::SetBounds(new, old),
        }
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::GridChange;
    use std::collections::HashMap;
    use tachy::geom::{Coords, CoordsRect, Direction, Orientation};
    use tachy::save::{ChipType, WireShape};

    fn collapse_group(changes: Vec<GridChange>) -> Vec<GridChange> {
        let inverted = GridChange::invert_and_collapse_group(changes);
        GridChange::invert_group(inverted)
    }

    fn wires(fragments: Vec<(Coords, Direction, WireShape)>)
             -> HashMap<(Coords, Direction), WireShape> {
        fragments.into_iter().map(|(c, d, s)| ((c, d), s)).collect()
    }

    #[test]
    fn collapse_empty_replace_wires_to_nothing() {
        let changes =
            vec![GridChange::ReplaceWires(wires(vec![]), wires(vec![]))];
        let expected = vec![];
        assert_eq!(collapse_group(changes), expected);
    }

    #[test]
    fn collapse_noop_replace_wires_to_nothing() {
        let fragment = (Coords::new(-2, 5), Direction::North, WireShape::Stub);
        let changes = vec![
            GridChange::ReplaceWires(wires(vec![fragment]),
                                     wires(vec![fragment])),
        ];
        let expected = vec![];
        assert_eq!(collapse_group(changes), expected);
    }

    #[test]
    fn collapse_inverse_replace_wires_to_nothing() {
        let coords = Coords::new(-2, 5);
        let changes = vec![
            GridChange::ReplaceWires(
                wires(vec![
                    (coords, Direction::East, WireShape::Straight),
                    (coords, Direction::West, WireShape::Straight),
                ]),
                wires(vec![
                    (coords, Direction::East, WireShape::Stub),
                    (coords, Direction::West, WireShape::Stub),
                ])
            ),
            GridChange::ReplaceWires(
                wires(vec![
                    (coords, Direction::East, WireShape::Stub),
                    (coords, Direction::West, WireShape::Stub),
                ]),
                wires(vec![
                    (coords, Direction::East, WireShape::Straight),
                    (coords, Direction::West, WireShape::Straight),
                ])
            ),
        ];
        let expected = vec![];
        assert_eq!(collapse_group(changes), expected);
    }

    #[test]
    fn collapse_two_replace_wires_into_one() {
        let coords = Coords::new(-2, 5);
        let changes = vec![
            GridChange::ReplaceWires(
                wires(vec![
                    (coords, Direction::East, WireShape::Straight),
                    (coords, Direction::West, WireShape::Straight),
                    (coords, Direction::North, WireShape::Stub),
                ]),
                wires(vec![
                    (coords, Direction::East, WireShape::Stub),
                    (coords, Direction::West, WireShape::TurnLeft),
                    (coords, Direction::North, WireShape::TurnRight),
                ])
            ),
            GridChange::ReplaceWires(
                wires(vec![
                    (coords, Direction::West, WireShape::TurnLeft),
                    (coords, Direction::North, WireShape::TurnRight),
                    (coords, Direction::South, WireShape::Stub),
                ]),
                wires(vec![
                    (coords, Direction::West, WireShape::TurnRight),
                    (coords, Direction::North, WireShape::Stub),
                    (coords, Direction::South, WireShape::TurnLeft),
                ])
            ),
        ];
        let expected = vec![
            GridChange::ReplaceWires(
                wires(vec![
                    (coords, Direction::East, WireShape::Straight),
                    (coords, Direction::West, WireShape::Straight),
                    (coords, Direction::South, WireShape::Stub),
                ]),
                wires(vec![
                    (coords, Direction::East, WireShape::Stub),
                    (coords, Direction::West, WireShape::TurnRight),
                    (coords, Direction::South, WireShape::TurnLeft),
                ])
            ),
        ];
        assert_eq!(collapse_group(changes), expected);
    }

    #[test]
    fn collapse_add_remove_chip() {
        let coords = Coords::new(3, 1);
        let orient1 = Orientation::default();
        let orient2 = orient1.rotate_cw();

        let changes =
            vec![
                GridChange::AddChip(coords, ChipType::Add, orient1),
                GridChange::RemoveChip(coords, ChipType::Add, orient1),
            ];
        let expected = vec![];
        assert_eq!(collapse_group(changes), expected);

        let changes =
            vec![
                GridChange::RemoveChip(coords, ChipType::Add, orient1),
                GridChange::AddChip(coords, ChipType::Add, orient1),
            ];
        let expected = vec![];
        assert_eq!(collapse_group(changes), expected);

        let changes =
            vec![
                GridChange::RemoveChip(coords, ChipType::Add, orient1),
                GridChange::AddChip(coords, ChipType::Add, orient2),
            ];
        let expected = changes.clone();
        assert_eq!(collapse_group(changes), expected);
    }

    #[test]
    fn collapse_set_bounds() {
        let rect1 = CoordsRect::new(0, 0, 8, 6);
        let rect2 = CoordsRect::new(1, 1, 7, 5);
        let rect3 = CoordsRect::new(1, 1, 9, 5);

        let changes = vec![
            GridChange::SetBounds(rect1, rect2),
            GridChange::SetBounds(rect2, rect3),
        ];
        let expected = vec![GridChange::SetBounds(rect1, rect3)];
        assert_eq!(collapse_group(changes), expected);

        let changes = vec![
            GridChange::SetBounds(rect1, rect2),
            GridChange::SetBounds(rect2, rect1),
        ];
        let expected = vec![];
        assert_eq!(collapse_group(changes), expected);

        let changes = vec![
            GridChange::SetBounds(rect1, rect2),
            GridChange::SetBounds(rect2, rect3),
            GridChange::SetBounds(rect3, rect1),
        ];
        let expected = vec![];
        assert_eq!(collapse_group(changes), expected);

        let changes = vec![
            GridChange::SetBounds(rect1, rect2),
            GridChange::SetBounds(rect2, rect2),
        ];
        let expected = vec![GridChange::SetBounds(rect1, rect2)];
        assert_eq!(collapse_group(changes), expected);

        let changes = vec![GridChange::SetBounds(rect1, rect1)];
        let expected = vec![];
        assert_eq!(collapse_group(changes), expected);
    }
}

//===========================================================================//
