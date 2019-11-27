extern crate tachy;

use std::fs;
use std::path::PathBuf;
use tachy::save::{CircuitData, Puzzle};
use tachy::state::EditGrid;

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

fn test_malformed_circuit(name: &str, puzzle: Puzzle) {
    let original_path =
        PathBuf::from(format!("tests/malformed/{}.original.toml", name));
    let original_data = CircuitData::load(&original_path).unwrap();
    let grid = EditGrid::from_circuit_data(
        puzzle,
        &Puzzle::all().collect(),
        &original_data,
    );
    let actual_repaired_string =
        grid.to_circuit_data().serialize_to_string().unwrap();
    let repaired_path =
        PathBuf::from(format!("tests/malformed/{}.repaired.toml", name));
    let expected_repaired_string = fs::read_to_string(&repaired_path).unwrap();
    assert_eq!(actual_repaired_string, expected_repaired_string);
}
