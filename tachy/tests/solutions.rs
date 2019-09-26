extern crate tachy;

use std::path::PathBuf;
use tachy::save::{CircuitData, Puzzle};
use tachy::state::{EditGrid, EvalResult};

#[test]
fn automate_heliostat_fast() {
    test_solution(Puzzle::AutomateHeliostat, "automate_heliostat_fast", 88);
}

#[test]
fn automate_heliostat_small() {
    test_solution(Puzzle::AutomateHeliostat, "automate_heliostat_small", 135);
}

#[test]
fn automate_sensors_fast() {
    test_solution(Puzzle::AutomateSensors, "automate_sensors_fast", 47);
}

#[test]
fn automate_sensors_small() {
    test_solution(Puzzle::AutomateSensors, "automate_sensors_small", 79);
}

#[test]
fn fabricate_halve() {
    test_solution(Puzzle::FabricateHalve, "fabricate_halve", 16);
}

#[test]
fn fabricate_mul() {
    test_solution(Puzzle::FabricateMul, "fabricate_mul", 10);
}

#[test]
fn fabricate_xor() {
    test_solution(Puzzle::FabricateXor, "fabricate_xor", 4);
}

#[test]
fn tutorial_add() {
    test_solution(Puzzle::TutorialAdd, "tutorial_add", 9);
}

#[test]
fn tutorial_mux() {
    test_solution(Puzzle::TutorialMux, "tutorial_mux", 8);
}

#[test]
fn tutorial_or() {
    test_solution(Puzzle::TutorialOr, "tutorial_or", 4);
}

fn test_solution(puzzle: Puzzle, name: &str, num_time_steps: u32) {
    let path = PathBuf::from(format!("tests/solutions/{}.toml", name));
    let data = CircuitData::load(&path).unwrap();
    let mut grid =
        EditGrid::from_circuit_data(puzzle, &Puzzle::all().collect(), &data);
    assert!(!grid.has_errors());
    assert!(grid.start_eval());
    let eval = grid.eval_mut().unwrap();
    for time_step in 0..(num_time_steps + 1) {
        match eval.step_time() {
            EvalResult::Continue if time_step < num_time_steps => {}
            EvalResult::Continue => {
                panic!("Evaluation did not end at time step {}", time_step);
            }
            EvalResult::Breakpoint(coords_list) => {
                panic!(
                    "Unexpected breakpoint at time step {}: {:?}",
                    time_step, coords_list
                );
            }
            EvalResult::Failure => {
                panic!(
                    "Unexpected failure at time step {}: {:?}",
                    time_step,
                    eval.errors()
                );
            }
            EvalResult::Victory(score) if time_step < num_time_steps => {
                panic!(
                    "Unexpected victory at time step {}: {:?}",
                    time_step, score
                );
            }
            EvalResult::Victory(_) => {}
        }
    }
}
