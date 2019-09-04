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

macro_rules! debug_log {
    ($e:expr) => {
        (if cfg!(debug_assertions) { eprintln!($e); })
    };
    ($fmt:expr, $($arg:tt)+) => {
        (if cfg!(debug_assertions) { eprintln!($fmt, $($arg)+); })
    };
}

macro_rules! debug_warn {
    ($($arg:tt)+) => {
        (if cfg!(debug_assertions) {
            eprint!("\x1b[31mWARNING:\x1b[m ");
            debug_log!($($arg)+);
        })
    };
}

//===========================================================================//
