extern crate tachy;

use tachy::save::SolutionData;
use tachy::state::verify_solution;

#[test]
fn cryocycler_already_thawed() {
    let actual = test_failure("cryocycler_already_thawed");
    let expected =
        vec!["Time step 36: Pod number 26 has already been thawed."];
    assert_eq!(actual, expected);
}

#[test]
fn cryocycler_invalid_pod() {
    let actual = test_failure("cryocycler_invalid_pod");
    let expected =
        vec!["Time step 0: Invalid pod number: 100 is not in the range 1-96."];
    assert_eq!(actual, expected);
}

#[test]
fn cryocycler_unready_pod() {
    let actual = test_failure("cryocycler_unready_pod");
    let expected = vec![
        "Time step 1: Pod number 1 is not ready this time step (the ready \
         pods are 22 and 30).",
    ];
    assert_eq!(actual, expected);
}

#[test]
fn fab_behavior_wrong() {
    let actual = test_failure("fab_behavior_wrong");
    let expected =
        vec!["Time step 3: Expected value of 0 for Out, but got value of 1."];
    assert_eq!(actual, expected);
}

#[test]
fn fuel_synth_intakes_open_while_mixing() {
    let actual = test_failure("fuel_synth_intakes_open_while_mixing");
    let expected = vec![
        "Time step 12: Intake valve #1 was left open during mixing.",
        "Time step 12: Intake valve #2 was left open during mixing.",
    ];
    assert_eq!(actual, expected);
}

#[test]
fn fuel_synth_mix_while_mixing() {
    let actual = test_failure("fuel_synth_mix_while_mixing");
    let expected =
        vec!["Time step 12: Tried to start mixing while already in progress."];
    assert_eq!(actual, expected);
}

#[test]
fn fuel_synth_not_enough_to_mix() {
    let actual = test_failure("fuel_synth_not_enough_to_mix");
    let expected = vec![
        "Time step 1: Tried to start mixing with not enough reagent in each \
         tank.",
    ];
    assert_eq!(actual, expected);
}

#[test]
fn fuel_synth_tank_1_overflow() {
    let actual = test_failure("fuel_synth_tank_1_overflow");
    let expected = vec![
        "Time step 19: Tank #1 is full, but the intake valve was left open.",
    ];
    assert_eq!(actual, expected);
}

#[test]
fn fuel_synth_tank_2_overflow() {
    let actual = test_failure("fuel_synth_tank_2_overflow");
    let expected = vec![
        "Time step 24: Tank #2 is full, but the intake valve was left open.",
    ];
    assert_eq!(actual, expected);
}

#[test]
fn infinite_loop() {
    let actual = test_failure("infinite_loop");
    let expected = vec!["Time step 0: Exceeded 1000 cycles."];
    assert_eq!(actual, expected);
}

#[test]
fn mining_robot_carry_too_much() {
    let actual = test_failure("mining_robot_carry_too_much");
    let expected = vec![
        "Time step 8: Tried to dig up a 3kg ore deposit \
         while already carrying 14kg (max load is 15kg).",
    ];
    assert_eq!(actual, expected);
}

#[test]
fn robot_arm_manipulate_twice() {
    let actual = test_failure("robot_arm_manipulate_twice");
    let expected = vec!["Time step 19: Already performed manipulation."];
    assert_eq!(actual, expected);
}

#[test]
fn robot_arm_manipulate_wrong_position() {
    let actual = test_failure("robot_arm_manipulate_wrong_position");
    let expected = vec![
        "Time step 0: Manipulated position 0, but last command was for \
         position 3.",
    ];
    assert_eq!(actual, expected);
}

#[test]
fn robot_arm_reply_before_done() {
    let actual = test_failure("robot_arm_reply_before_done");
    let expected = vec![
        "Time step 0: Sent radio reply without first completing the \
         instructed manipulation.",
    ];
    assert_eq!(actual, expected);
}

#[test]
fn robot_arm_reply_twice() {
    let actual = test_failure("robot_arm_reply_twice");
    let expected = vec![
        "Time step 19: Sent more than one radio reply for the same command.",
    ];
    assert_eq!(actual, expected);
}

#[test]
fn sonar_crash_port() {
    let actual = test_failure("sonar_crash_port");
    let expected =
        vec!["Time step 63: Crashed into the port-side canyon wall."];
    assert_eq!(actual, expected);
}

#[test]
fn sonar_crash_stbd() {
    let actual = test_failure("sonar_crash_stbd");
    let expected =
        vec!["Time step 8: Crashed into the starboard-side canyon wall."];
    assert_eq!(actual, expected);
}

#[test]
fn x_unit_incomplete_detonation() {
    let actual = test_failure("x_unit_incomplete_detonation");
    let expected =
        vec!["Time step 5: Only 4 out of 256 charges were detonated at once."];
    assert_eq!(actual, expected);
}

fn test_failure(name: &str) -> Vec<String> {
    let path = format!("tests/failures/{}.toml", name);
    let data = SolutionData::load(&path).unwrap();
    verify_solution(&data)
}
