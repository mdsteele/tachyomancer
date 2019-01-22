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

use super::eval::{CircuitState, EvalError, EvalScore, NullPuzzleEval,
                  PuzzleEval};
use cgmath::Point2;
use num_integer::Roots;
use tachy::geom::{Coords, Direction};
use tachy::save::Puzzle;

//===========================================================================//

pub fn new_puzzle_eval(puzzle: Puzzle,
                       slots: Vec<Vec<((Coords, Direction), usize)>>)
                       -> Box<PuzzleEval> {
    match puzzle {
        Puzzle::TutorialOr => Box::new(TutorialOrEval::new(slots)),
        Puzzle::AutomateHeliostat => {
            Box::new(AutomateHeliostatEval::new(slots))
        }
        Puzzle::SandboxEvent => Box::new(SandboxEventEval::new(slots)),
        _ => Box::new(NullPuzzleEval()), // TODO other puzzles
    }
}

//===========================================================================//

struct TutorialOrEval {
    verification: Vec<u64>,
    input1_wire: usize,
    input2_wire: usize,
    output_wire: usize,
    output_port: (Coords, Direction),
}

impl TutorialOrEval {
    fn new(slots: Vec<Vec<((Coords, Direction), usize)>>) -> TutorialOrEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        TutorialOrEval {
            verification: Puzzle::TutorialOr
                .static_verification_data()
                .to_vec(),
            input1_wire: slots[0][0].1,
            input2_wire: slots[1][0].1,
            output_wire: slots[2][0].1,
            output_port: slots[2][0].0,
        }
    }
}

impl PuzzleEval for TutorialOrEval {
    fn verification_data(&self) -> &[u64] { &self.verification }

    fn begin_time_step(&mut self, time_step: u32, state: &mut CircuitState)
                       -> Option<EvalScore> {
        if time_step >= 4 {
            Some(EvalScore::WireLength)
        } else {
            state.send_behavior(self.input1_wire, time_step & 0x1);
            state.send_behavior(self.input2_wire, (time_step & 0x2) >> 1);
            None
        }
    }

    fn end_time_step(&mut self, time_step: u32, state: &CircuitState)
                     -> Vec<EvalError> {
        let input1 = state.recv_behavior(self.input1_wire).0;
        let input2 = state.recv_behavior(self.input2_wire).0;
        let expected = input1 | input2;
        let actual = state.recv_behavior(self.output_wire).0;
        self.verification[3 * (time_step as usize) + 2] = actual as u64;
        if actual != expected {
            let error = EvalError {
                time_step,
                port: Some(self.output_port),
                message: format!("Expected output {} for inputs {} and {}, \
                                  but output was {}",
                                 expected,
                                 input1,
                                 input2,
                                 actual),
            };
            vec![error]
        } else {
            vec![]
        }
    }
}

//===========================================================================//

struct AutomateHeliostatEval {
    verification: [u64; 5],
    opt_x_wire: usize,
    opt_y_wire: usize,
    pos_x_wire: usize,
    pos_y_wire: usize,
    motor_wire: usize,
    current_opt: Point2<u32>,
    current_pos: Point2<u32>,
    energy: u32,
    rng: SimpleRng,
}

impl AutomateHeliostatEval {
    fn new(slots: Vec<Vec<((Coords, Direction), usize)>>)
           -> AutomateHeliostatEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 3);
        AutomateHeliostatEval {
            verification: [0; 5],
            opt_x_wire: slots[0][0].1,
            opt_y_wire: slots[0][1].1,
            pos_x_wire: slots[1][0].1,
            pos_y_wire: slots[1][1].1,
            motor_wire: slots[1][2].1,
            current_opt: Point2::new(3, 7),
            current_pos: Point2::new(15, 15),
            energy: 1000,
            rng: SimpleRng::new(0x4f3173b1f817227f),
        }
    }

    fn update_verification(&mut self) {
        self.verification[0] = self.current_opt.x as u64;
        self.verification[1] = self.current_opt.y as u64;
        self.verification[2] = self.current_pos.x as u64;
        self.verification[3] = self.current_pos.y as u64;
        self.verification[4] = self.energy as u64;
    }
}

impl PuzzleEval for AutomateHeliostatEval {
    fn verification_data(&self) -> &[u64] { &self.verification }

    fn begin_time_step(&mut self, time_step: u32, state: &mut CircuitState)
                       -> Option<EvalScore> {
        if (time_step % 20) == 0 {
            let x = self.rng.rand_u4();
            let y = self.rng.rand_u4();
            self.current_opt = Point2::new(x, y);
        }
        self.update_verification();
        state.send_behavior(self.opt_x_wire, self.current_opt.x);
        state.send_behavior(self.opt_y_wire, self.current_opt.y);
        state.send_behavior(self.pos_x_wire, self.current_pos.x);
        state.send_behavior(self.pos_y_wire, self.current_pos.y);
        if self.energy >= 5000 {
            Some(EvalScore::Value(time_step as i32))
        } else {
            None
        }
    }

    fn end_time_step(&mut self, _time_step: u32, state: &CircuitState)
                     -> Vec<EvalError> {
        let delta = 10 *
            Point2::new(self.current_pos.x as i32 - self.current_opt.x as i32,
                        self.current_pos.y as i32 - self.current_opt.y as i32);
        let dist = (delta.x * delta.x + delta.y * delta.y).sqrt() as u32;
        self.energy += 85;
        self.energy = self.energy.saturating_sub(dist);
        match state.recv_behavior(self.motor_wire).0 {
            0x8 if self.current_pos.y < 0xf => self.current_pos.y += 1,
            0x4 if self.current_pos.y > 0x0 => self.current_pos.y -= 1,
            0x2 if self.current_pos.x > 0x0 => self.current_pos.x -= 1,
            0x1 if self.current_pos.x < 0xf => self.current_pos.x += 1,
            _ => {}
        }
        self.update_verification();
        Vec::new()
    }
}

//===========================================================================//

struct SandboxEventEval {
    metronome: usize,
    timer: usize,
}

impl SandboxEventEval {
    fn new(slots: Vec<Vec<((Coords, Direction), usize)>>) -> SandboxEventEval {
        debug_assert_eq!(slots.len(), 1);
        debug_assert_eq!(slots[0].len(), 2);
        SandboxEventEval {
            metronome: slots[0][0].1,
            timer: slots[0][1].1,
        }
    }
}

impl PuzzleEval for SandboxEventEval {
    fn verification_data(&self) -> &[u64] { &[] }

    fn begin_time_step(&mut self, time_step: u32, state: &mut CircuitState)
                       -> Option<EvalScore> {
        state.send_event(self.metronome, 0);
        state.send_behavior(self.timer, time_step & 0xff);
        None
    }
}

//===========================================================================//

/// A simple, not-very-good pseudo-random number generator.  We use this
/// instead of the rand crate because (1) we don't need a very good RNG, but
/// (2) we do need the random sequence to be deterministic and guaranteed
/// stable across compiles and crate versions.
struct SimpleRng {
    z: u32,
    w: u32,
}

impl SimpleRng {
    fn new(seed: u64) -> SimpleRng {
        SimpleRng {
            z: 0x159a55e5 ^ ((seed & 0xffffffff) as u32),
            w: 0x1f123bb5 ^ ((seed >> 32) as u32),
        }
    }

    fn rand_u32(&mut self) -> u32 {
        // This RNG is based on the MWC algorithm from George Marsaglia's post
        // to sci.stat.math on 12 Jan 1999, which can be found here:
        //   https://groups.google.com/forum/#!topic/sci.stat.math/5yb0jwf1stw
        self.z = 36969 * (self.z & 0xffff) + (self.z >> 16);
        self.w = 18000 * (self.w & 0xffff) + (self.w >> 16);
        (self.z << 16) | (self.w & 0xffff)
    }

    fn rand_u4(&mut self) -> u32 { self.rand_u32() & 0xf }
}

//===========================================================================//
