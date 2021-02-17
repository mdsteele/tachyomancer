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

use tachy::save::SolutionData;
use tachy::state::verify_solution;

//===========================================================================//

#[test]
fn automate_collector() {
    test_solution("automate_collector");
}

#[test]
fn automate_cryocycler() {
    test_solution("automate_cryocycler");
}

#[test]
fn automate_drilling_rig() {
    test_solution("automate_drilling_rig");
}

#[test]
fn automate_enrichment() {
    test_solution("automate_enrichment");
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
fn automate_resonator_fast() {
    test_solution("automate_resonator_fast");
}

#[test]
fn automate_resonator_small() {
    test_solution("automate_resonator_small");
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
fn automate_sonar() {
    test_solution("automate_sonar");
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
fn command_sapper_manual() {
    test_solution("command_sapper_manual");
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
fn fabricate_latch() {
    test_solution("fabricate_latch");
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
fn tutorial_adc() {
    test_solution("tutorial_adc");
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

//===========================================================================//

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

//===========================================================================//
