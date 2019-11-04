extern crate tachy;

use std::path::PathBuf;
use tachy::save::SolutionData;
use tachy::state::verify_solution;

#[test]
fn infinite_loop() {
    let actual = test_failure("infinite_loop");
    let expected = vec!["Time step 0: Exceeded 1000 cycles"];
    assert_eq!(actual, expected);
}

fn test_failure(name: &str) -> Vec<String> {
    let path = PathBuf::from(format!("tests/failures/{}.toml", name));
    let data = SolutionData::load(&path).unwrap();
    verify_solution(&data)
}
