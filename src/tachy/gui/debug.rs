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

use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;

//===========================================================================//

pub struct StdinReader {
    queue: Arc<Mutex<VecDeque<String>>>,
}

impl StdinReader {
    pub(super) fn start() -> StdinReader {
        let queue = Arc::new(Mutex::new(VecDeque::<String>::new()));
        let stdin_reader = StdinReader { queue: queue.clone() };
        thread::spawn(move || {
            let reader = io::stdin();
            loop {
                let mut string = String::new();
                match reader.read_line(&mut string) {
                    Err(err) => {
                        debug_log!("ERROR: StdinReader failed: {}", err);
                        return;
                    }
                    Ok(0) => {
                        debug_log!("StdinReader reached EOF.");
                        return;
                    }
                    Ok(_) => {
                        queue.lock().unwrap().push_back(string);
                    }
                }
            }
        });
        stdin_reader
    }

    pub fn pop_line(&mut self) -> Option<String> {
        self.queue.lock().unwrap().pop_front()
    }
}

//===========================================================================//
