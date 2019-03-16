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

use png;
use sdl2::mouse::{self, SystemCursor};
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;
use std::collections::HashMap;

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
const CURSOR_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/texture/cursor.png"));

const NUM_CURSOR_COLS: usize = 4;
const NUM_CURSOR_ROWS: usize = 4;

// Table of (cursor, (col, row), hotspot).
#[cfg_attr(rustfmt, rustfmt_skip)]
const CURSORS: &[(Cursor, (usize, usize), (i32, i32))] = &[
    (Cursor::Arrow,                    (0, 0), (1, 1)),
    (Cursor::Crosshair,                (1, 0), (7, 7)),
    (Cursor::ResizeEastWest,           (0, 1), (7, 7)),
    (Cursor::ResizeNorthSouth,         (1, 1), (7, 7)),
    (Cursor::ResizeNortheastSouthwest, (2, 1), (7, 7)),
    (Cursor::ResizeNorthwestSoutheast, (3, 1), (7, 7)),
];

//===========================================================================//

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Cursor {
    Arrow,
    Crosshair,
    HandClosed,
    HandOpen,
    HandPointing,
    ResizeEastWest,
    ResizeNorthSouth,
    ResizeNortheastSouthwest,
    ResizeNorthwestSoutheast,
    Text,
    Wire,
}

impl Default for Cursor {
    fn default() -> Cursor { Cursor::Arrow }
}

//===========================================================================//

pub struct Cursors {
    current_cursor: Cursor,
    cursors: HashMap<Cursor, mouse::Cursor>,
}

impl Cursors {
    pub(super) fn new() -> Result<Cursors, String> {
        let png_name = "texture/cursor";
        let decoder = png::Decoder::new(CURSOR_PNG_DATA);
        let (info, mut reader) = decoder
            .read_info()
            .map_err(|err| {
                         format!("Failed to read PNG header for {}: {}",
                                 png_name,
                                 err)
                     })?;

        // Determine pixel format:
        if info.bit_depth != png::BitDepth::Eight {
            return Err(format!("PNG {} bit depth should be {:?}, but is {:?}",
                               png_name,
                               png::BitDepth::Eight,
                               info.bit_depth));
        }
        if info.color_type != png::ColorType::RGBA {
            return Err(format!("PNG {} color type should be {:?}, but is {:?}",
                               png_name,
                               png::ColorType::RGBA,
                               info.color_type));
        }
        let bytes_per_pixel: usize = 4;
        let sdl_format = if cfg!(target_endian = "big") {
            PixelFormatEnum::RGBA8888
        } else {
            PixelFormatEnum::ABGR8888
        };

        // Determine dimensions:
        let total_width = info.width as usize;
        let total_height = info.height as usize;
        if total_width % NUM_CURSOR_COLS != 0 {
            return Err(format!("PNG {} width should be a multiple of {}, \
                                but is {}",
                               png_name,
                               NUM_CURSOR_COLS,
                               total_width));
        }
        if total_height % NUM_CURSOR_ROWS != 0 {
            return Err(format!("PNG {} height should be a multiple of {}, \
                                but is {}",
                               png_name,
                               NUM_CURSOR_ROWS,
                               total_height));
        }
        let cursor_width = total_width / NUM_CURSOR_COLS;
        let cursor_height = total_height / NUM_CURSOR_ROWS;

        // Read data:
        let mut data = vec![0u8; bytes_per_pixel * total_width * total_height];
        if let Err(err) = reader.next_frame(&mut data) {
            return Err(format!("Failed to decode PNG data for {}: {}",
                               png_name,
                               err));
        }

        // Construct SDL cursors:
        let mut cursors = HashMap::<Cursor, mouse::Cursor>::new();
        for &(cursor, (col, row), (hot_x, hot_y)) in CURSORS.iter() {
            let data_start = col * cursor_width +
                row * total_width * cursor_height;
            let sdl_surface =
                Surface::from_data(&mut data[(bytes_per_pixel *
                                                  data_start)..],
                                   cursor_width as u32,
                                   cursor_height as u32,
                                   (bytes_per_pixel * total_width) as u32,
                                   sdl_format)?;
            let sdl_cursor =
                mouse::Cursor::from_surface(&sdl_surface, hot_x, hot_y)?;
            cursors.insert(cursor, sdl_cursor);
        }
        // TODO: Use custom cursors for these:
        cursors.insert(Cursor::HandClosed,
                       mouse::Cursor::from_system(SystemCursor::SizeAll)?);
        cursors.insert(Cursor::HandOpen,
                       mouse::Cursor::from_system(SystemCursor::Hand)?);
        cursors.insert(Cursor::HandPointing,
                       mouse::Cursor::from_system(SystemCursor::Hand)?);
        cursors.insert(Cursor::Text,
                       mouse::Cursor::from_system(SystemCursor::IBeam)?);
        cursors.insert(Cursor::Wire,
                       mouse::Cursor::from_system(SystemCursor::Arrow)?);
        let current_cursor = Cursor::default();
        cursors[&current_cursor].set();
        Ok(Cursors {
               current_cursor,
               cursors,
           })
    }

    pub fn set(&mut self, next: NextCursor) {
        if let Some(cursor) = next.requested {
            if cursor != self.current_cursor {
                self.cursors[&cursor].set();
                self.current_cursor = cursor;
            }
        }
    }
}

//===========================================================================//

pub struct NextCursor {
    requested: Option<Cursor>,
}

impl NextCursor {
    pub fn new() -> NextCursor { NextCursor { requested: None } }

    pub fn request(&mut self, cursor: Cursor) {
        if self.requested.is_none() {
            self.requested = Some(cursor);
        }
    }
}

//===========================================================================//
