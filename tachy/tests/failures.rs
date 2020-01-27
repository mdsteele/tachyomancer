extern crate tachy;

use std::path::PathBuf;
use tachy::save::SolutionData;
use tachy::state::verify_solution;

#[test]
fn fab_behavior_wrong() {
    let actual = test_failure("fab_behavior_wrong");
    let expected =
        vec!["Time step 3: Expected value of 0 for Out, but got value of 1."];
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
fn x_unit_incomplete_detonation() {
    let actual = test_failure("x_unit_incomplete_detonation");
    let expected =
        vec!["Time step 5: Only 4 out of 256 charges were detonated at once."];
    assert_eq!(actual, expected);
}

fn test_failure(name: &str) -> Vec<String> {
    let path = PathBuf::from(format!("tests/failures/{}.toml", name));
    let data = SolutionData::load(&path).unwrap();
    verify_solution(&data)
}
