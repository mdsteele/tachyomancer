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

use cgmath::Matrix4;
use tachy::geom::{Color4, Rect};
use tachy::gl::{IndexBuffer, Primitive, Shader, ShaderProgram, ShaderType,
                ShaderUniform, Texture2D, VertexArray, VertexBuffer};

//===========================================================================//

const UI_VERT_CODE: &[u8] = include_bytes!("ui.vert");
const UI_FRAG_CODE: &[u8] = include_bytes!("ui.frag");
#[cfg_attr(rustfmt, rustfmt_skip)]
const UI_TEXTURE_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/texture/ui.png"));

#[cfg_attr(rustfmt, rustfmt_skip)]
const INDEX_DATA: &[u8] = &[
    0, 1, 4,  4, 1, 5,
    1, 2, 5,  5, 2, 6,
    2, 3, 6,  6, 3, 7,

    4, 5, 8,  8, 5, 9,
    5, 6, 9,  9, 6, 10,
    6, 7, 10,  10, 7, 11,

    8, 9, 12,  12, 9, 13,
    9, 10, 13,  13, 10, 14,
    10, 11, 14,  14, 11, 15,
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const CORNERS_DATA: &[u8] = &[
    0, 0,  0, 0,  1, 0,  1, 0,
    0, 0,  0, 0,  1, 0,  1, 0,
    0, 1,  0, 1,  1, 1,  1, 1,
    0, 1,  0, 1,  1, 1,  1, 1,
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const CHECKBOX_DATA: &[f32] = &[
    0.0, 0.0, 0.0, 0.0,  0.0, 0.0, 0.0, 0.0,
    1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,

    0.0, 0.0, 0.0, 0.0,  0.0, 0.0, 0.0, 0.0,
    1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,

    0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,
    1.0, 1.0, 0.0, 0.0,  1.0, 1.0, 0.0, 0.0,

    0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,
    1.0, 1.0, 0.0, 0.0,  1.0, 1.0, 0.0, 0.0,
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const DIALOG_DATA: &[f32] = &[
    0.0, 0.0, 0.0, 0.0,  0.25, 0.0, 64.0, 0.0,
    0.375, 0.0, -32.0, 0.0,  0.5, 0.0, 0.0, 0.0,

    0.0, 0.125, 0.0, 32.0,  0.25, 0.125, 64.0, 32.0,
    0.375, 0.125, -32.0, 32.0,  0.5, 0.125, 0.0, 32.0,

    0.0, 0.25, 0.0, -64.0,  0.25, 0.25, 64.0, -64.0,
    0.375, 0.25, -32.0, -64.0,  0.5, 0.25, 0.0, -64.0,

    0.0, 0.5, 0.0, 0.0,  0.25, 0.5, 64.0, 0.0,
    0.375, 0.5, -32.0, 0.0,  0.5, 0.5, 0.0, 0.0,
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const RECT4_DATA: &[f32] = &[
    0.5, 0.0, 0.0, 0.0,  0.575, 0.0, 19.2, 0.0,
    0.675, 0.0, -19.2, 0.0,  0.75, 0.0, 0.0, 0.0,

    0.5, 0.075, 0.0, 19.2,  0.575, 0.075, 19.2, 19.2,
    0.675, 0.075, -19.2, 19.2,  0.75, 0.075, 0.0, 19.2,

    0.5, 0.175, 0.0, -19.2,  0.575, 0.175, 19.2, -19.2,
    0.675, 0.175, -19.2, -19.2,  0.75, 0.175, 0.0, -19.2,

    0.5, 0.25, 0.0, 0.0,  0.575, 0.25, 19.2, 0.0,
    0.675, 0.25, -19.2, 0.0,  0.75, 0.25, 0.0, 0.0,
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const SCROLL_DATA: &[f32] = &[
    0.0, 0.0, 0.0, 0.0,  0.25, 0.0, 8.0, 0.0,
    0.75, 0.0, -8.0, 0.0,  1.0, 0.0, 0.0, 0.0,

    0.0, 0.25, 0.0, 8.0,  0.25, 0.25, 8.0, 8.0,
    0.75, 0.25, -8.0, 8.0,  1.0, 0.25, 0.0, 8.0,

    0.0, 0.75, 0.0, -8.0,  0.25, 0.75, 8.0, -8.0,
    0.75, 0.75, -8.0, -8.0,  1.0, 0.75, 0.0, -8.0,

    0.0, 1.0, 0.0, 0.0,  0.25, 1.0, 8.0, 0.0,
    0.75, 1.0, -8.0, 0.0,  1.0, 1.0, 0.0, 0.0,
];

//===========================================================================//

pub struct UiShader {
    program: ShaderProgram,
    texture: Texture2D,
    mvp: ShaderUniform<Matrix4<f32>>,
    screen_rect: ShaderUniform<Rect<f32>>,
    tex_rect: ShaderUniform<Rect<f32>>,
    color1: ShaderUniform<Color4>,
    color2: ShaderUniform<Color4>,
    color3: ShaderUniform<Color4>,
    ibuffer: IndexBuffer<u8>,
    _corners_vbuffer: VertexBuffer<u8>,
    _checkbox_vbuffer: VertexBuffer<f32>,
    checkbox_varray: VertexArray,
    _dialog_vbuffer: VertexBuffer<f32>,
    dialog_varray: VertexArray,
    _rect4_vbuffer: VertexBuffer<f32>,
    rect4_varray: VertexArray,
    _scroll_vbuffer: VertexBuffer<f32>,
    scroll_varray: VertexArray,
}

impl UiShader {
    pub(super) fn new() -> Result<UiShader, String> {
        let vert = Shader::new(ShaderType::Vertex, "ui.vert", UI_VERT_CODE)?;
        let frag = Shader::new(ShaderType::Fragment, "ui.frag", UI_FRAG_CODE)?;
        let program = ShaderProgram::new(&[&vert, &frag])?;
        let texture = Texture2D::from_png("texture/ui", UI_TEXTURE_PNG_DATA)?;
        let mvp = program.get_uniform("MVP")?;
        let screen_rect = program.get_uniform("ScreenRect")?;
        let tex_rect = program.get_uniform("TexRect")?;
        let color1 = program.get_uniform("Color1")?;
        let color2 = program.get_uniform("Color2")?;
        let color3 = program.get_uniform("Color3")?;
        let ibuffer = IndexBuffer::new(INDEX_DATA);
        let corners_vbuffer = VertexBuffer::new(CORNERS_DATA);

        let (checkbox_varray, checkbox_vbuffer) =
            make_vertices(&corners_vbuffer, CHECKBOX_DATA);
        let (dialog_varray, dialog_vbuffer) = make_vertices(&corners_vbuffer,
                                                            DIALOG_DATA);
        let (rect4_varray, rect4_vbuffer) = make_vertices(&corners_vbuffer,
                                                          RECT4_DATA);
        let (scroll_varray, scroll_vbuffer) = make_vertices(&corners_vbuffer,
                                                            SCROLL_DATA);

        let shader = UiShader {
            program,
            texture,
            mvp,
            screen_rect,
            tex_rect,
            color1,
            color2,
            color3,
            ibuffer,
            _corners_vbuffer: corners_vbuffer,
            _checkbox_vbuffer: checkbox_vbuffer,
            checkbox_varray,
            _dialog_vbuffer: dialog_vbuffer,
            dialog_varray,
            _rect4_vbuffer: rect4_vbuffer,
            rect4_varray,
            _scroll_vbuffer: scroll_vbuffer,
            scroll_varray,
        };
        Ok(shader)
    }

    fn bind(&self, matrix: &Matrix4<f32>, screen_rect: &Rect<f32>,
            color1: &Color4, color2: &Color4, color3: &Color4,
            tex_rect: &Rect<f32>) {
        self.program.bind();
        self.texture.bind();
        self.mvp.set(matrix);
        self.screen_rect.set(screen_rect);
        self.tex_rect.set(tex_rect);
        self.color1.set(color1);
        self.color2.set(color2);
        self.color3.set(color3);
    }

    pub fn draw_checkbox(&self, matrix: &Matrix4<f32>, rect: &Rect<f32>,
                         color1: &Color4, color2: &Color4, color3: &Color4,
                         checked: bool) {
        let tex_rect = if checked {
            Rect::new(0.875, 0.125, 0.125, 0.125)
        } else {
            Rect::new(0.75, 0.125, 0.125, 0.125)
        };
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.checkbox_varray.bind();
        self.checkbox_varray
            .draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_dialog(&self, matrix: &Matrix4<f32>, rect: &Rect<f32>,
                       color1: &Color4, color2: &Color4, color3: &Color4) {
        let tex_rect = Rect::new(0.0, 0.0, 1.0, 1.0);
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.dialog_varray.bind();
        self.dialog_varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_rect4(&self, matrix: &Matrix4<f32>, rect: &Rect<f32>,
                      color1: &Color4, color2: &Color4, color3: &Color4) {
        let tex_rect = Rect::new(0.0, 0.0, 1.0, 1.0);
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.rect4_varray.bind();
        self.rect4_varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_scroll_bar(&self, matrix: &Matrix4<f32>, rect: &Rect<f32>,
                           color1: &Color4, color2: &Color4, color3: &Color4) {
        let tex_rect = Rect::new(0.875, 0.0, 0.125, 0.125);
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.scroll_varray.bind();
        self.scroll_varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_scroll_handle(&self, matrix: &Matrix4<f32>,
                              rect: &Rect<f32>, color1: &Color4,
                              color2: &Color4, color3: &Color4) {
        let tex_rect = Rect::new(0.75, 0.0, 0.125, 0.125);
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.scroll_varray.bind();
        self.scroll_varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }
}

fn make_vertices(corners: &VertexBuffer<u8>, data: &[f32])
                 -> (VertexArray, VertexBuffer<f32>) {
    let varray = VertexArray::new(3);
    let vbuffer = VertexBuffer::new(data);
    varray.bind();
    corners.attribi(0, 2, 0, 0);
    vbuffer.attribf(1, 2, 4, 0);
    vbuffer.attribf(2, 2, 4, 2);
    (varray, vbuffer)
}

//===========================================================================//
