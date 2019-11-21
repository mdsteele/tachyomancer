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

extern crate heck;
extern crate icns;
extern crate nsvg;
extern crate png;
extern crate rusttype;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use heck::{ShoutySnakeCase, TitleCase};
use png::HasParameters;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

//===========================================================================//

fn main() {
    let converter = Converter::new();
    converter.assemble_resource_info("font", &["src/mancer/font"]);
    converter.assemble_resource_info("music", &["src/mancer/music"]);
    converter.assemble_resource_info("sound", &["src/mancer/sound"]);
    converter.assemble_resource_info(
        "texture",
        &[
            "src/mancer/texture",
            "src/mancer/texture/portrait",
            "src/mancer/texture/scene",
        ],
    );
    converter.font_to_texture("galactico", true);
    converter.font_to_texture("inconsolata-bold", true);
    converter.font_to_texture("inconsolata-regular", true);
    converter.font_to_texture("segment7", false);
    converter.generate_chip_icons();
    converter.generate_list_icons();
    converter.generate_portraits_texture();
    converter.svg_to_png(
        &PathBuf::from("src/mancer/gui/cursor.svg"),
        &PathBuf::from("texture/cursor.png"),
        icns::PixelFormat::RGBA,
    );
    converter.svg_to_png(
        &PathBuf::from("src/mancer/shader/ui.svg"),
        &PathBuf::from("texture/ui.png"),
        icns::PixelFormat::RGBA,
    );
    converter.svgs_to_pngs(
        &PathBuf::from("src/mancer/texture/diagram"),
        &PathBuf::from("texture/diagram"),
        icns::PixelFormat::RGBA,
    );

    let target = env::var("TARGET").unwrap();
    if target.ends_with("-apple-darwin") {
        println!("cargo:rustc-link-search=framework=/Library/Frameworks");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
}

//===========================================================================//

const GLYPH_HIRES_SIZE: usize = 64;
const DIST_SPREAD: usize = 6;

const CHIP_ICON_SIZE: usize = 64;
const CHIP_TEXTURE_COLS: usize = 8;
const CHIP_TEXTURE_ROWS: usize = 8;

const LIST_ICON_SIZE: usize = 32;
const LIST_TEXTURE_COLS: usize = 4;
const LIST_TEXTURE_ROWS: usize = 4;

const PORTRAIT_WIDTH: usize = 68;
const PORTRAIT_HEIGHT: usize = 85;
const PORTRAITS_TEXTURE_WIDTH: usize = 256;
const PORTRAITS_TEXTURE_HEIGHT: usize = 256;

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
        Converter { build_script_timestamp, out_dir }
    }

    fn assemble_resource_info(&self, name: &str, dirs: &[&str]) {
        let mut infos = Vec::<ResourceInfo>::new();
        for dir in dirs {
            for entry in PathBuf::from(dir).read_dir().unwrap() {
                let path = entry.unwrap().path();
                if let Some(name) = path.file_name() {
                    let name = name.to_str().unwrap();
                    if name.ends_with(".info.toml") || name == "info.toml" {
                        let contents = fs::read(path).unwrap();
                        let info = toml::from_slice(&contents).unwrap();
                        infos.push(info);
                    }
                }
            }
        }
        infos.sort_by_key(|info| info.name.clone());
        let out_relpath = format!("rsrc_info/{}.rs", name);
        eprintln!("Generating: {:?}", out_relpath);
        let out_path = self.out_dir.join(out_relpath);
        self.create_parent_dir(&out_path);
        let mut out_file = File::create(&out_path).unwrap();
        writeln!(
            out_file,
            "const {}_RESOURCE_INFO: &[ResourceInfo] = &[",
            name.to_shouty_snake_case()
        )
        .unwrap();
        for info in infos.iter() {
            writeln!(out_file, "    ResourceInfo {{").unwrap();
            writeln!(out_file, "        name: {:?},", info.name).unwrap();
            writeln!(out_file, "        artist: {:?},", info.artist).unwrap();
            writeln!(out_file, "        license: {:?},", info.license)
                .unwrap();
            writeln!(out_file, "        year: {},", info.year).unwrap();
            writeln!(out_file, "        url: {:?},", info.url).unwrap();
            writeln!(out_file, "    }},").unwrap();
        }
        writeln!(out_file, "];").unwrap();
    }

    fn font_to_texture(&self, font_name: &str, centered: bool) {
        // Check if the output PNG file is already up-to-date:
        let font_path =
            PathBuf::from(format!("src/mancer/font/{}.ttf", font_name));
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
            let x =
                ((vmetrics.ascent - vmetrics.descent) / bounds.width()) * y;
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
                let mut xoff = ((codepoint % 16) as i32)
                    * (GLYPH_HIRES_SIZE as i32)
                    + (DIST_SPREAD as i32);
                let xpadding = (GLYPH_HIRES_SIZE as i32)
                    - 2 * (DIST_SPREAD as i32)
                    - bounds.width();
                if centered {
                    xoff += xpadding / 2;
                } else {
                    xoff += xpadding;
                }
                let yoff = ((codepoint / 16) as i32)
                    * (GLYPH_HIRES_SIZE as i32)
                    + (DIST_SPREAD as i32)
                    + ascent
                    + bounds.min.y;
                positioned.draw(|x, y, v| {
                    let value = (255.0 * v) as u8;
                    let x = xoff + (x as i32);
                    let y = yoff + (y as i32);
                    if (x >= 0 && (x as usize) < 16 * GLYPH_HIRES_SIZE)
                        && (y >= 0 && (y as usize) < 16 * GLYPH_HIRES_SIZE)
                    {
                        let index = (x as usize)
                            + (y as usize) * 16 * GLYPH_HIRES_SIZE;
                        hires[index] = value;
                    }
                });
            }
        }

        // Write the output PNG file:
        let png_file = File::create(&png_path).unwrap();
        let mut encoder = png::Encoder::new(
            png_file,
            16 * GLYPH_HIRES_SIZE as u32,
            16 * GLYPH_HIRES_SIZE as u32,
        );
        encoder.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&hires).unwrap();
    }

    fn generate_chip_icons(&self) {
        // Convert chip icon SVGs to PNGs:
        let png_paths = self.svgs_to_pngs(
            &PathBuf::from("src/mancer/texture/chip"),
            &PathBuf::from("chip"),
            icns::PixelFormat::Alpha,
        );
        let mut icon_names: Vec<String> = png_paths
            .iter()
            .map(|path| {
                path.file_stem().unwrap().to_str().unwrap().to_title_case()
            })
            .collect();
        icon_names.sort();

        // Combine icon PNGs into a single texture PNG:
        self.sprite_images(
            (CHIP_ICON_SIZE, CHIP_ICON_SIZE),
            icns::PixelFormat::Alpha,
            &png_paths,
            (
                CHIP_ICON_SIZE * CHIP_TEXTURE_COLS,
                CHIP_ICON_SIZE * CHIP_TEXTURE_ROWS,
            ),
            icns::PixelFormat::Gray,
            &PathBuf::from("texture/chip_icons.png"),
        );

        // Generate ChipIcon enum:
        let icon_rs_relpath = PathBuf::from("texture/chip_icons.rs");
        eprintln!("Generating: {:?}", icon_rs_relpath);
        let icon_rs_path = self.out_dir.join(&icon_rs_relpath);
        let mut icon_rs = File::create(&icon_rs_path).unwrap();
        writeln!(icon_rs, "#[derive(Clone, Copy, Eq, PartialEq)]").unwrap();
        writeln!(icon_rs, "enum ChipIcon {{").unwrap();
        for (index, icon_name) in icon_names.iter().enumerate() {
            writeln!(icon_rs, "    {} = {},", icon_name, index).unwrap();
        }
        writeln!(icon_rs, "    Blank = {},", icon_names.len()).unwrap();
        writeln!(icon_rs, "}}").unwrap();
    }

    fn generate_list_icons(&self) {
        // Convert list icon SVGs to PNGs:
        let png_paths = self.svgs_to_pngs(
            &PathBuf::from("src/mancer/texture/listicon"),
            &PathBuf::from("listicon"),
            icns::PixelFormat::Alpha,
        );
        let mut icon_names: Vec<String> = png_paths
            .iter()
            .map(|path| {
                path.file_stem().unwrap().to_str().unwrap().to_title_case()
            })
            .collect();
        icon_names.sort();

        // Combine icon PNGs into a single texture PNG:
        self.sprite_images(
            (LIST_ICON_SIZE, LIST_ICON_SIZE),
            icns::PixelFormat::Alpha,
            &png_paths,
            (
                LIST_ICON_SIZE * LIST_TEXTURE_COLS,
                LIST_ICON_SIZE * LIST_TEXTURE_ROWS,
            ),
            icns::PixelFormat::Gray,
            &PathBuf::from("texture/list_icons.png"),
        );

        // Generate ListIcon enum:
        let icon_rs_relpath = PathBuf::from("texture/list_icons.rs");
        eprintln!("Generating: {:?}", icon_rs_relpath);
        let icon_rs_path = self.out_dir.join(&icon_rs_relpath);
        let mut icon_rs = File::create(&icon_rs_path).unwrap();
        writeln!(icon_rs, "#[derive(Clone, Copy, Eq, PartialEq)]").unwrap();
        writeln!(icon_rs, "pub enum ListIcon {{").unwrap();
        for (index, icon_name) in icon_names.iter().enumerate() {
            writeln!(icon_rs, "    {} = {},", icon_name, index).unwrap();
        }
        writeln!(icon_rs, "}}").unwrap();
    }

    fn generate_portraits_texture(&self) {
        let png_dir = PathBuf::from("src/mancer/texture/portrait");
        let mut png_paths = Vec::<PathBuf>::new();
        for entry in png_dir.read_dir().unwrap() {
            let png_path = entry.unwrap().path();
            if png_path.extension() != Some("png".as_ref()) {
                continue;
            }
            png_paths.push(png_path);
        }
        png_paths.sort();
        self.sprite_images(
            (PORTRAIT_WIDTH, PORTRAIT_HEIGHT),
            icns::PixelFormat::Gray,
            &png_paths,
            (PORTRAITS_TEXTURE_WIDTH, PORTRAITS_TEXTURE_HEIGHT),
            icns::PixelFormat::Gray,
            &PathBuf::from("texture/portraits.png"),
        );
    }

    fn sprite_images(
        &self,
        (png_width, png_height): (usize, usize),
        png_format: icns::PixelFormat,
        png_paths: &[PathBuf],
        (texture_width, texture_height): (usize, usize),
        texture_format: icns::PixelFormat,
        texture_relpath: &Path,
    ) {
        // Find the latest input timestamp:
        let mut latest_timestamp = self.build_script_timestamp;
        for png_path in png_paths.iter() {
            let png_timestamp =
                png_path.metadata().unwrap().modified().unwrap();
            latest_timestamp = latest_timestamp.max(png_timestamp);
        }

        // Check whether the output file is already up-to-date:
        let texture_path = self.out_dir.join(texture_relpath);
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

        // Combine the input PNGs into a single texture PNG:
        let mut texture_data = vec![0u8; texture_width * texture_height];
        let num_texture_cols = texture_width / png_width;
        for (index, png_path) in png_paths.iter().enumerate() {
            let texture_col = index % num_texture_cols;
            let texture_row = index / num_texture_cols;
            let png_file = File::open(&png_path).unwrap();
            let mut png_image = icns::Image::read_png(png_file).unwrap();
            if png_image.pixel_format() != png_format {
                png_image = png_image.convert_to(png_format);
            }
            assert_eq!(png_image.width() as usize, png_width);
            assert_eq!(png_image.height() as usize, png_height);
            let png_data = png_image.data();
            for y in 0..png_height {
                let src_start = y * png_width;
                let src_slice = &png_data[src_start..(src_start + png_width)];
                let dst_start = texture_row * texture_width * png_height
                    + texture_col * png_width
                    + y * texture_width;
                let dst_slice =
                    &mut texture_data[dst_start..(dst_start + png_width)];
                dst_slice.copy_from_slice(src_slice);
            }
        }
        let texture_image = icns::Image::from_data(
            texture_format,
            texture_width as u32,
            texture_height as u32,
            texture_data,
        )
        .unwrap();
        texture_image.write_png(File::create(texture_path).unwrap()).unwrap();
    }

    fn svg_to_png(
        &self,
        svg_path: &Path,
        png_relpath: &Path,
        pixel_format: icns::PixelFormat,
    ) -> PathBuf {
        // Check if the output PNG file is already up-to-date:
        let png_path = self.out_dir.join(png_relpath);
        if self.is_up_to_date(&png_path, svg_path) {
            eprintln!("Up-to-date: {:?}", png_relpath);
            return png_path;
        }
        eprintln!("Generating: {:?}", png_relpath);
        self.create_parent_dir(&png_path);

        // Convert the SVG to PNG:
        let svg =
            nsvg::parse_file(svg_path, nsvg::Units::Pixel, 96.0).unwrap();
        let (width, height, rgba) = svg.rasterize_to_raw_rgba(1.0).unwrap();

        let mut image = icns::Image::from_data(
            icns::PixelFormat::RGBA,
            width,
            height,
            rgba,
        )
        .unwrap();
        if pixel_format != icns::PixelFormat::RGBA {
            image = image.convert_to(pixel_format);
        }
        image.write_png(File::create(&png_path).unwrap()).unwrap();
        return png_path;
    }

    fn svgs_to_pngs(
        &self,
        svg_dir_path: &Path,
        png_dir_relpath: &Path,
        pixel_format: icns::PixelFormat,
    ) -> Vec<PathBuf> {
        let mut png_paths = Vec::<PathBuf>::new();
        for entry in svg_dir_path.read_dir().unwrap() {
            let svg_path = entry.unwrap().path();
            if svg_path.extension() != Some("svg".as_ref()) {
                continue;
            }
            let svg_name = svg_path.file_stem().unwrap();
            let png_relpath =
                png_dir_relpath.join(svg_name).with_extension("png");
            let png_path =
                self.svg_to_png(&svg_path, &png_relpath, pixel_format);
            png_paths.push(png_path);
        }
        png_paths.sort();
        png_paths
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

#[derive(Deserialize, Serialize)]
struct ResourceInfo {
    name: String,
    artist: String,
    license: String,
    year: i32,
    url: String,
}

//===========================================================================//
