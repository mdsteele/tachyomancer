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

extern crate png;
extern crate rusttype;

use png::HasParameters;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

//===========================================================================//

fn main() {
    let build_script_timestamp =
        fs::metadata("build.rs").unwrap().modified().unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    font_to_texture(build_script_timestamp, &out_dir, "inconsolata");
}

//===========================================================================//

const GLYPH_HIRES_SIZE: usize = 64;
const DIST_SPREAD: usize = 6;

fn font_to_texture(build_script_timestamp: SystemTime, out_dir: &Path,
                   font_name: &str) {
    // Check if the output PNG file is already up-to-date:
    let font_path = PathBuf::from(format!("src/tachy/font/{}.ttf", font_name));
    let png_dir = out_dir.join("font");
    let png_path = png_dir.join(format!("{}.png", font_name));
    if png_path.is_file() {
        let png_timestamp = png_path.metadata().unwrap().modified().unwrap();
        if png_timestamp >= build_script_timestamp &&
            png_timestamp >=
                font_path.metadata().unwrap().modified().unwrap()
        {
            eprintln!("Up-to-date: font/{}.png", font_name);
            return;
        }
    }
    eprintln!("Generating: font/{}.png", font_name);
    fs::create_dir_all(&png_dir).unwrap();

    // Load the input TTF file and determine metrics:
    let font_data = fs::read(&font_path).unwrap();
    let font = rusttype::Font::from_bytes(font_data).unwrap();
    let scale = {
        let y = (GLYPH_HIRES_SIZE - 2 * DIST_SPREAD) as f32;
        let vmetrics = font.v_metrics(rusttype::Scale::uniform(y));
        let glyph = font.glyph(rusttype::Codepoint(b'W' as u32));
        let scaled = glyph.scaled(rusttype::Scale::uniform(y));
        let bounds = scaled.exact_bounding_box().unwrap();
        let x = ((vmetrics.ascent - vmetrics.descent) / bounds.width()) * y;
        rusttype::Scale { x, y }
    };
    let ascent = font.v_metrics(scale).ascent.ceil() as i32;

    // Render glyphs at high resolution:
    let mut hires = vec![0u8; 256 * GLYPH_HIRES_SIZE * GLYPH_HIRES_SIZE];
    for codepoint in 0..256 {
        let glyph = font.glyph(rusttype::Codepoint(codepoint));
        let scaled = glyph.scaled(scale);
        let positioned = scaled.positioned(rusttype::Point { x: 0.0, y: 0.0 });
        if let Some(bounds) = positioned.pixel_bounding_box() {
            let xoff = ((codepoint % 16) as i32) * (GLYPH_HIRES_SIZE as i32) +
                (DIST_SPREAD as i32) +
                ((GLYPH_HIRES_SIZE as i32) - 2 * (DIST_SPREAD as i32) -
                     bounds.width()) / 2;
            let yoff = ((codepoint / 16) as i32) * (GLYPH_HIRES_SIZE as i32) +
                (DIST_SPREAD as i32) + ascent +
                bounds.min.y;
            positioned.draw(|x, y, v| {
                let value = (255.0 * v) as u8;
                let x = xoff + (x as i32);
                let y = yoff + (y as i32);
                if (x >= 0 && (x as usize) < 16 * GLYPH_HIRES_SIZE) &&
                    (y >= 0 && (y as usize) < 16 * GLYPH_HIRES_SIZE)
                {
                    let index = (x as usize) +
                        (y as usize) * 16 * GLYPH_HIRES_SIZE;
                    hires[index] = value;
                }
            });
        }
    }

    // Write the output PNG file:
    let png_file = File::create(&png_path).unwrap();
    let mut encoder = png::Encoder::new(png_file,
                                        16 * GLYPH_HIRES_SIZE as u32,
                                        16 * GLYPH_HIRES_SIZE as u32);
    encoder.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&hires).unwrap();
}

//===========================================================================//
