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

mod enums;

pub use self::enums::{Align, Font};
use cgmath::{Matrix4, Vector2};
use std::rc::Rc;
use tachy::geom::{Color4, MatrixExt};
use tachy::gl::{Primitive, Shader, ShaderProgram, ShaderSampler, ShaderType,
                ShaderUniform, Texture2D, VertexArray, VertexBuffer};

//===========================================================================//

const MAX_CHARS: usize = 64;

#[cfg_attr(rustfmt, rustfmt_skip)]
const CHAR_VERTICES: &[(u8, u8)] = &[
    (0, 0), (0, 1), (1, 0),
    (1, 0), (0, 1), (1, 1),
];

const TEXT_VERT_CODE: &[u8] = include_bytes!("text.vert");
const TEXT_FRAG_CODE: &[u8] = include_bytes!("text.frag");

//===========================================================================//

pub struct Fonts {
    alien: FontData,
    bold: FontData,
    roman: FontData,
}

impl Fonts {
    pub fn new() -> Result<Fonts, String> {
        let shader = Rc::new(TextShader::new()?);
        let alien = FontData::new(Font::Alien, &shader)?;
        let bold = FontData::new(Font::Bold, &shader)?;
        let roman = FontData::new(Font::Roman, &shader)?;
        Ok(Fonts { alien, bold, roman })
    }

    pub fn get(&self, font: Font) -> &FontData {
        match font {
            Font::Alien => self.alien(),
            Font::Bold => self.bold(),
            Font::Roman => self.roman(),
        }
    }

    pub fn alien(&self) -> &FontData { &self.alien }

    pub fn bold(&self) -> &FontData { &self.bold }

    pub fn roman(&self) -> &FontData { &self.roman }
}

//===========================================================================//

pub struct FontData {
    shader: Rc<TextShader>,
    texture: Texture2D,
    ratio: f32,
}

impl FontData {
    fn new(font: Font, shader: &Rc<TextShader>) -> Result<FontData, String> {
        let (png_name, png_data) = font.png_name_and_data();
        let texture = Texture2D::from_png(png_name, png_data)?;
        Ok(FontData {
               shader: shader.clone(),
               texture,
               ratio: font.ratio(),
           })
    }

    pub fn ratio(&self) -> f32 { self.ratio }

    pub fn str_width(&self, height: f32, text: &str) -> f32 {
        Font::str_width_for_ratio(self.ratio(), height, text)
    }

    pub fn draw(&self, matrix: &Matrix4<f32>, height: f32, align: Align,
                start: (f32, f32), text: &str) {
        let color = &Color4::WHITE;
        let slant = 0.0;
        self.draw_style(matrix, height, align, start, color, slant, text);
    }

    pub fn draw_style(&self, matrix: &Matrix4<f32>, height: f32,
                      align: Align, start: (f32, f32), color: &Color4,
                      slant: f32, text: &str) {
        let chars: Vec<u8> = text.chars().map(|chr| chr as u8).collect();
        self.draw_chars(matrix, height, align, start, color, slant, &chars);
    }

    pub fn draw_chars(&self, matrix: &Matrix4<f32>, height: f32,
                      align: Align, start: (f32, f32), color: &Color4,
                      slant: f32, chars: &[u8]) {
        let size = (self.ratio * height, height);
        self.shader.draw(matrix,
                         size,
                         align,
                         start,
                         color,
                         slant,
                         &self.texture,
                         chars);
    }
}

//===========================================================================//

struct TextShader {
    program: ShaderProgram,
    color: ShaderUniform<Color4>,
    mvp: ShaderUniform<Matrix4<f32>>,
    text: ShaderUniform<[u32; MAX_CHARS]>,
    slant: ShaderUniform<f32>,
    font: ShaderSampler<Texture2D>,
    varray: VertexArray,
    _vbuffer: VertexBuffer<u8>,
}

impl TextShader {
    fn new() -> Result<TextShader, String> {
        // Create shader:
        let vert =
            Shader::new(ShaderType::Vertex, "text.vert", TEXT_VERT_CODE)?;
        let frag =
            Shader::new(ShaderType::Fragment, "text.frag", TEXT_FRAG_CODE)?;
        let program = ShaderProgram::new(&[&vert, &frag])?;
        let color = program.get_uniform("Color")?;
        let mvp = program.get_uniform("MVP")?;
        let text = program.get_uniform("Text")?;
        let slant = program.get_uniform("Slant")?;
        let font = program.get_sampler(0, "Font")?;

        // Set up vertex data:
        let varray = VertexArray::new(2);
        let data_len = 3 * CHAR_VERTICES.len() * MAX_CHARS;
        let mut data = Vec::<u8>::with_capacity(data_len);
        for index in 0..MAX_CHARS {
            for &(x, y) in CHAR_VERTICES.iter() {
                data.push(x);
                data.push(y);
                data.push(index as u8);
            }
        }
        debug_assert_eq!(data.len(), data_len);
        let vbuffer = VertexBuffer::new(&data);
        varray.bind();
        vbuffer.attribi(0, 2, 3, 0);
        vbuffer.attribi(1, 1, 3, 2);

        let shader = TextShader {
            program,
            color,
            mvp,
            text,
            slant,
            font,
            varray,
            _vbuffer: vbuffer,
        };
        Ok(shader)
    }

    fn draw(&self, matrix: &Matrix4<f32>, size: (f32, f32),
            alignment: Align, start: (f32, f32), color: &Color4, slant: f32,
            font: &Texture2D, chars: &[u8]) {
        self.program.bind();
        self.varray.bind();
        self.color.set(color);
        self.slant.set(&slant);
        self.font.set(font);

        let num_chars = chars.len();
        let mut shift = match alignment {
            Align::TopLeft => Vector2::new(0.0, 0.0),
            Align::MidLeft => Vector2::new(0.0, -0.5),
            Align::TopCenter => Vector2::new(-0.5 * (num_chars as f32), 0.0),
            Align::MidCenter => Vector2::new(-0.5 * (num_chars as f32), -0.5),
            Align::BottomCenter => {
                Vector2::new(-0.5 * (num_chars as f32), -1.0)
            }
            Align::TopRight => Vector2::new(-(num_chars as f32), 0.0),
            Align::MidRight => Vector2::new(-(num_chars as f32), -0.5),
        };

        let mut array = [0u32; MAX_CHARS];
        let mut offset = 0;
        while offset < num_chars {
            let stride = (num_chars - offset).min(MAX_CHARS);
            for i in 0..stride {
                array[i] = chars[offset + i] as u32;
            }
            self.text.set(&array);
            let mvp = matrix * Matrix4::trans2(start.0, start.1) *
                Matrix4::scale2(size.0, size.1) *
                Matrix4::trans2v(shift);
            self.mvp.set(&mvp);
            self.varray
                .draw(Primitive::Triangles, 0, CHAR_VERTICES.len() * stride);
            shift.x += stride as f32;
            offset += stride;
        }
    }
}

//===========================================================================//
