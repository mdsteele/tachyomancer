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

use cgmath::{Matrix4, Vector2, Vector3};
use std::rc::Rc;
use tachy::geom::MatrixExt;
use tachy::gl::{Primitive, Shader, ShaderProgram, ShaderType, ShaderUniform,
                Texture2D, VertexArray, VertexBuffer};

//===========================================================================//

const BLACK_COLOR: (f32, f32, f32) = (0.0, 0.0, 0.0);
const MAX_CHARS: usize = 64;

#[cfg_attr(rustfmt, rustfmt_skip)]
const CHAR_VERTICES: &[(u8, u8)] = &[
    (0, 0), (0, 1), (1, 0),
    (1, 0), (0, 1), (1, 1),
];

const TEXT_VERT_CODE: &[u8] = include_bytes!("text.vert");
const TEXT_FRAG_CODE: &[u8] = include_bytes!("text.frag");

#[cfg_attr(rustfmt, rustfmt_skip)]
const INCONSOLATA_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/font/inconsolata.png"));

// ========================================================================= //

#[derive(Clone, Copy)]
pub enum Align {
    TopLeft,
    MidLeft,
    TopCenter,
    MidCenter,
    BottomCenter,
    TopRight,
    MidRight,
}

//===========================================================================//

pub struct Fonts {
    roman: Font,
}

impl Fonts {
    pub fn new() -> Result<Fonts, String> {
        let shader = Rc::new(TextShader::new()?);
        let roman =
            Font::new("font/inconsolata", INCONSOLATA_PNG_DATA, 0.5, &shader)?;
        Ok(Fonts { roman })
    }

    pub fn roman(&self) -> &Font { &self.roman }
}

//===========================================================================//

pub struct Font {
    shader: Rc<TextShader>,
    texture: Texture2D,
    ratio: f32,
}

impl Font {
    fn new(png_name: &str, png_data: &[u8], ratio: f32,
           shader: &Rc<TextShader>)
           -> Result<Font, String> {
        let texture = Texture2D::from_png(png_name, png_data)?;
        Ok(Font {
               shader: shader.clone(),
               texture,
               ratio,
           })
    }

    pub fn draw(&self, matrix: &Matrix4<f32>, height: f32, alignment: Align,
                start: (f32, f32), text: &str) {
        self.draw_color(matrix, height, alignment, start, BLACK_COLOR, text);
    }

    pub fn draw_color(&self, matrix: &Matrix4<f32>, height: f32,
                      alignment: Align, start: (f32, f32),
                      color: (f32, f32, f32), text: &str) {
        self.texture.bind();
        let size = (self.ratio * height, height);
        self.shader.draw(matrix, size, alignment, start, &color.into(), text);
    }
}

//===========================================================================//

struct TextShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    text: ShaderUniform<[u32; MAX_CHARS]>,
    text_color: ShaderUniform<Vector3<f32>>,
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
        let mvp = program.get_uniform("MVP")?;
        let text = program.get_uniform("Text")?;
        let text_color = program.get_uniform("TextColor")?;

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

        Ok(TextShader {
               program,
               mvp,
               text,
               text_color,
               varray,
               _vbuffer: vbuffer,
           })
    }

    fn draw(&self, matrix: &Matrix4<f32>, size: (f32, f32),
            alignment: Align, start: (f32, f32), color: &Vector3<f32>,
            text: &str) {
        self.program.bind();
        self.varray.bind();
        self.text_color.set(color);

        let chars: Vec<u32> = text.chars().map(|c| c as u32).collect();
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
            (&mut array[..stride])
                .copy_from_slice(&chars[offset..(offset + stride)]);
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
