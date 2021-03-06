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

extern crate tachy;

use std::fs;
use std::path::PathBuf;
use tachy::save::{CircuitData, Puzzle, PuzzleSet};
use tachy::state::EditGrid;

//===========================================================================//

#[test]
fn chip_not_allowed() {
    test_malformed_circuit("disallowed_or_gate", Puzzle::TutorialOr);
}

#[test]
fn chip_out_of_bounds() {
    test_malformed_circuit("chip_out_of_bounds", Puzzle::SandboxEvent);
}

#[test]
fn empty_bounds() {
    test_malformed_circuit("empty_bounds", Puzzle::SandboxEvent);
}

#[test]
fn mismatched_wires() {
    test_malformed_circuit("mismatched_wires", Puzzle::SandboxEvent);
}

#[test]
fn overlapping_chips() {
    test_malformed_circuit("overlapping_chips", Puzzle::SandboxEvent);
}

#[test]
fn wires_out_of_bounds() {
    test_malformed_circuit("wires_out_of_bounds", Puzzle::TutorialOr);
}

#[test]
fn wires_underneath_chips() {
    test_malformed_circuit("wires_underneath_chips", Puzzle::SandboxEvent);
}

//===========================================================================//

fn test_malformed_circuit(name: &str, puzzle: Puzzle) {
    let original_path =
        PathBuf::from(format!("tests/malformed/{}.original.toml", name));
    let original_data = CircuitData::load(&original_path).unwrap();
    let grid = EditGrid::from_circuit_data(
        puzzle,
        &PuzzleSet::with_everything_solved(),
        &original_data,
    );
    let actual_repaired_string =
        grid.to_circuit_data().serialize_to_string().unwrap();
    let repaired_path =
        PathBuf::from(format!("tests/malformed/{}.repaired.toml", name));
    let expected_repaired_string = fs::read_to_string(&repaired_path).unwrap();
    assert_eq!(actual_repaired_string, expected_repaired_string);
}

//===========================================================================//
