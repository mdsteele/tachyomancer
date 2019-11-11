extern crate tachy;

use std::path::PathBuf;
use tachy::save::SolutionData;
use tachy::state::verify_solution;

#[test]
fn automate_grapple() {
    test_solution("automate_grapple");
}

#[test]
fn automate_heliostat_fast() {
    test_solution("automate_heliostat_fast");
}

#[test]
fn automate_heliostat_small() {
    test_solution("automate_heliostat_small");
}

#[test]
fn automate_mining_robot() {
    test_solution("automate_mining_robot");
}

#[test]
fn automate_sensors_fast() {
    test_solution("automate_sensors_fast");
}

#[test]
fn automate_sensors_small() {
    test_solution("automate_sensors_small");
}

#[test]
fn automate_storage_depot() {
    test_solution("automate_storage_depot");
}

#[test]
fn automate_x_unit() {
    test_solution("automate_x_unit");
}

#[test]
fn fabricate_egg_timer() {
    test_solution("fabricate_egg_timer");
}

#[test]
fn fabricate_halve() {
    test_solution("fabricate_halve");
}

#[test]
fn fabricate_mul() {
    test_solution("fabricate_mul");
}

#[test]
fn fabricate_stopwatch() {
    test_solution("fabricate_stopwatch");
}

#[test]
fn fabricate_xor() {
    test_solution("fabricate_xor");
}

#[test]
fn tutorial_add() {
    test_solution("tutorial_add");
}

#[test]
fn tutorial_mux() {
    test_solution("tutorial_mux");
}

#[test]
fn tutorial_or() {
    test_solution("tutorial_or");
}

fn test_solution(name: &str) {
    let path = PathBuf::from(format!("tests/solutions/{}.toml", name));
    let data = SolutionData::load(&path).unwrap();
    let errors = verify_solution(&data);
    if !errors.is_empty() {
        for error in errors {
            eprintln!("Error: {}", error);
        }
        panic!("Solution had errors");
    }
}
