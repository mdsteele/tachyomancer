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

extern crate icns;
extern crate nsvg;
extern crate png;
extern crate rusttype;

use png::HasParameters;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

//===========================================================================//

fn main() {
    let converter = Converter::new();
    converter.font_to_texture("galactico");
    converter.font_to_texture("inconsolata-bold");
    converter.font_to_texture("inconsolata-regular");
    converter.generate_chip_icons();
    converter.svg_to_png(&PathBuf::from("src/tachy/gui/cursor.svg"),
                         &PathBuf::from("texture/cursor.png"),
                         icns::PixelFormat::RGBA);
    converter.svg_to_png(&PathBuf::from("src/tachy/shader/ui.svg"),
                         &PathBuf::from("texture/ui.png"),
                         icns::PixelFormat::RGBA);
}

//===========================================================================//

const GLYPH_HIRES_SIZE: usize = 64;
const DIST_SPREAD: usize = 6;

const CHIP_ICON_SIZE: usize = 64;
const CHIP_TEXTURE_COLS: usize = 8;
const CHIP_TEXTURE_ROWS: usize = 8;

struct Converter {
    build_script_timestamp: SystemTime,
    out_dir: PathBuf,
}

impl Converter {
    fn new() -> Converter {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        eprintln!("OUT_DIR={:?}", out_dir);
        let build_script_timestamp =
            fs::metadata("build.rs").unwrap().modified().unwrap();
        Converter {
            build_script_timestamp,
            out_dir,
        }
    }

    fn font_to_texture(&self, font_name: &str) {
        // Check if the output PNG file is already up-to-date:
        let font_path = PathBuf::from(format!("src/tachy/font/{}.ttf",
                                              font_name));
        let png_relpath = PathBuf::from(format!("font/{}.png", font_name));
        let png_path = self.out_dir.join(&png_relpath);
        if self.is_up_to_date(&png_path, &font_path) {
            eprintln!("Up-to-date: {:?}", png_relpath);
            return;
        }
        eprintln!("Generating: {:?}", png_relpath);
        self.create_parent_dir(&png_path);

        // Load the input TTF file and determine metrics:
        let font_data = fs::read(&font_path).unwrap();
        let font = rusttype::Font::from_bytes(font_data).unwrap();
        let scale = {
            let y = (GLYPH_HIRES_SIZE - 2 * DIST_SPREAD) as f32;
            let vmetrics = font.v_metrics(rusttype::Scale::uniform(y));
            let glyph = font.glyph(rusttype::Codepoint(b'W' as u32));
            let scaled = glyph.scaled(rusttype::Scale::uniform(y));
            let bounds = scaled.exact_bounding_box().unwrap();
            let x = ((vmetrics.ascent - vmetrics.descent) / bounds.width()) *
                y;
            rusttype::Scale { x, y }
        };
        let ascent = font.v_metrics(scale).ascent.ceil() as i32;

        // Render glyphs at high resolution:
        let mut hires = vec![0u8; 256 * GLYPH_HIRES_SIZE * GLYPH_HIRES_SIZE];
        for codepoint in 0..256 {
            let glyph = font.glyph(rusttype::Codepoint(codepoint));
            let scaled = glyph.scaled(scale);
            let positioned =
                scaled.positioned(rusttype::Point { x: 0.0, y: 0.0 });
            if let Some(bounds) = positioned.pixel_bounding_box() {
                let xoff = ((codepoint % 16) as i32) *
                    (GLYPH_HIRES_SIZE as i32) +
                    (DIST_SPREAD as i32) +
                    ((GLYPH_HIRES_SIZE as i32) - 2 * (DIST_SPREAD as i32) -
                         bounds.width()) / 2;
                let yoff = ((codepoint / 16) as i32) *
                    (GLYPH_HIRES_SIZE as i32) +
                    (DIST_SPREAD as i32) +
                    ascent + bounds.min.y;
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

    fn generate_chip_icons(&self) {
        // Convert chip icon SVGs to PNGs:
        let svg_dir = PathBuf::from("src/tachy/texture/chip");
        let mut latest_timestamp = self.build_script_timestamp;
        let mut png_paths = Vec::<PathBuf>::new();
        for entry in svg_dir.read_dir().unwrap() {
            let svg_path = entry.unwrap().path();
            if svg_path.extension() != Some("svg".as_ref()) {
                continue;
            }
            let svg_name = svg_path.file_stem().unwrap();
            let png_relpath =
                PathBuf::from("chip").join(svg_name).with_extension("png");
            let png_path =
                self.svg_to_png(&svg_path,
                                &png_relpath,
                                icns::PixelFormat::Alpha);
            let png_timestamp =
                png_path.metadata().unwrap().modified().unwrap();
            latest_timestamp = latest_timestamp.max(png_timestamp);
            png_paths.push(png_path);
        }

        // Check if the output PNG file is already up-to-date:
        let texture_relpath = PathBuf::from("texture/chip_icons.png");
        let texture_path = self.out_dir.join(&texture_relpath);
        if texture_path.is_file() {
            let texture_timestamp =
                texture_path.metadata().unwrap().modified().unwrap();
            if texture_timestamp >= latest_timestamp {
                eprintln!("Up-to-date: {:?}", texture_relpath);
                return;
            }
        }
        eprintln!("Generating: {:?}", texture_relpath);
        self.create_parent_dir(&texture_path);

        // Combine icon PNGs into a single texture PNG:
        png_paths.sort();
        let texture_width = CHIP_ICON_SIZE * CHIP_TEXTURE_COLS;
        let texture_height = CHIP_ICON_SIZE * CHIP_TEXTURE_ROWS;
        let mut texture_data = vec![0u8; texture_width * texture_height];
        for (index, png_path) in png_paths.iter().enumerate() {
            let texture_col = index % CHIP_TEXTURE_COLS;
            let texture_row = index / CHIP_TEXTURE_COLS;
            let png_file = File::open(&png_path).unwrap();
            let icon = icns::Image::read_png(png_file).unwrap();
            let icon = icon.convert_to(icns::PixelFormat::Alpha);
            assert_eq!(icon.width() as usize, CHIP_ICON_SIZE);
            assert_eq!(icon.height() as usize, CHIP_ICON_SIZE);
            let icon_data = icon.data();
            for y in 0..CHIP_ICON_SIZE {
                let src_start = y * CHIP_ICON_SIZE;
                let src_slice = &icon_data[src_start..
                                               (src_start + CHIP_ICON_SIZE)];
                let dst_start = texture_row * texture_width * CHIP_ICON_SIZE +
                    texture_col * CHIP_ICON_SIZE +
                    y * texture_width;
                let dst_slice =
                    &mut texture_data[dst_start..(dst_start + CHIP_ICON_SIZE)];
                dst_slice.copy_from_slice(src_slice);
            }
        }
        let texture_image = icns::Image::from_data(icns::PixelFormat::Gray,
                                                   texture_width as u32,
                                                   texture_height as u32,
                                                   texture_data)
            .unwrap();
        texture_image.write_png(File::create(&texture_path).unwrap()).unwrap();
    }

    fn svg_to_png(&self, svg_path: &Path, png_relpath: &Path,
                  pixel_format: icns::PixelFormat)
                  -> PathBuf {
        // Check if the output PNG file is already up-to-date:
        let png_path = self.out_dir.join(png_relpath);
        if self.is_up_to_date(&png_path, svg_path) {
            eprintln!("Up-to-date: {:?}", png_relpath);
            return png_path;
        }
        eprintln!("Generating: {:?}", png_relpath);
        self.create_parent_dir(&png_path);

        // Convert the SVG to PNG:
        let svg = nsvg::parse_file(svg_path, nsvg::Units::Pixel, 96.0)
            .unwrap();
        let (width, height, rgba) = svg.rasterize_to_raw_rgba(1.0).unwrap();

        let mut image = icns::Image::from_data(icns::PixelFormat::RGBA,
                                               width,
                                               height,
                                               rgba)
            .unwrap();
        if pixel_format != icns::PixelFormat::RGBA {
            image = image.convert_to(pixel_format);
        }
        image.write_png(File::create(&png_path).unwrap()).unwrap();
        return png_path;
    }

    fn create_parent_dir(&self, path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
    }

    fn is_up_to_date(&self, out_path: &Path, in_path: &Path) -> bool {
        if out_path.is_file() {
            let out_timestamp =
                out_path.metadata().unwrap().modified().unwrap();
            if out_timestamp >= self.build_script_timestamp {
                let in_timestamp =
                    in_path.metadata().unwrap().modified().unwrap();
                return out_timestamp >= in_timestamp;
            }
        }
        return false;
    }
}

//===========================================================================//
