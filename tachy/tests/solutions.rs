extern crate tachy;

use tachy::save::SolutionData;
use tachy::state::verify_solution;

#[test]
fn automate_collector() {
    test_solution("automate_collector");
}

#[test]
fn automate_drilling_rig() {
    test_solution("automate_drilling_rig");
}

#[test]
fn automate_fuel_synth() {
    test_solution("automate_fuel_synth");
}

#[test]
fn automate_geiger_counter_fast() {
    test_solution("automate_geiger_counter_fast");
}

#[test]
fn automate_geiger_counter_small() {
    test_solution("automate_geiger_counter_small");
}

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
fn automate_incubator() {
    test_solution("automate_incubator");
}

#[test]
fn automate_injector() {
    test_solution("automate_injector");
}

#[test]
fn automate_mining_robot() {
    test_solution("automate_mining_robot");
}

#[test]
fn automate_reactor_fast() {
    test_solution("automate_reactor_fast");
}

#[test]
fn automate_reactor_medium() {
    test_solution("automate_reactor_medium");
}

#[test]
fn automate_reactor_small() {
    test_solution("automate_reactor_small");
}

#[test]
fn automate_robot_arm() {
    test_solution("automate_robot_arm");
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
fn automate_translator() {
    test_solution("automate_translator");
}

#[test]
fn automate_x_unit() {
    test_solution("automate_x_unit");
}

#[test]
fn command_lander_auto() {
    test_solution("command_lander_auto");
}

#[test]
fn command_lander_manual() {
    test_solution("command_lander_manual");
}

#[test]
fn command_shields_manual() {
    test_solution("command_shields_manual");
}

#[test]
fn fabricate_counter() {
    test_solution("fabricate_counter");
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
fn fabricate_inc() {
    test_solution("fabricate_inc");
}

#[test]
fn fabricate_mul() {
    test_solution("fabricate_mul");
}

#[test]
fn fabricate_queue() {
    test_solution("fabricate_queue");
}

#[test]
fn fabricate_stack() {
    test_solution("fabricate_stack");
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
fn tutorial_amp() {
    test_solution("tutorial_amp");
}

#[test]
fn tutorial_clock() {
    test_solution("tutorial_clock");
}

#[test]
fn tutorial_demux() {
    test_solution("tutorial_demux");
}

#[test]
fn tutorial_mux() {
    test_solution("tutorial_mux");
}

#[test]
fn tutorial_or() {
    test_solution("tutorial_or");
}

#[test]
fn tutorial_ram() {
    test_solution("tutorial_ram");
}

#[test]
fn tutorial_sum() {
    test_solution("tutorial_sum");
}

fn test_solution(name: &str) {
    let path = format!("tests/solutions/{}.toml", name);
    let data = SolutionData::load(&path).unwrap();
    let errors = verify_solution(&data);
    if !errors.is_empty() {
        for error in errors {
            eprintln!("Error: {}", error);
        }
        panic!("Solution had errors");
    }
}
