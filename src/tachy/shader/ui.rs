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

use crate::tachy::geom::{Color4, MatrixExt, Rect};
use crate::tachy::gl::{
    IndexBuffer, Primitive, Shader, ShaderProgram, ShaderSampler, ShaderType,
    ShaderUniform, Texture2D, VertexArray, VertexBuffer,
};
use cgmath::Matrix4;
use num_integer::div_mod_floor;

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
const BOX_DATA: &[f32] = &[
    0.0, 0.0, 0.0, 0.0,  0.3125, 0.0, 20.0, 0.0,
    0.6875, 0.0, -20.0, 0.0,  1.0, 0.0, 0.0, 0.0,

    0.0, 0.5, 0.0, 16.0,  0.3125, 0.5, 20.0, 16.0,
    0.6875, 0.5, -20.0, 16.0,  1.0, 0.5, 0.0, 16.0,

    0.0, 0.5, 0.0, -16.0,  0.3125, 0.5, 20.0, -16.0,
    0.6875, 0.5, -20.0, -16.0,  1.0, 0.5, 0.0, -16.0,

    0.0, 1.0, 0.0, 0.0,  0.3125, 1.0, 20.0, 0.0,
    0.6875, 1.0, -20.0, 0.0,  1.0, 1.0, 0.0, 0.0,
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const BUBBLE_DATA: &[f32] = &[
    0.0, 0.0, 0.0, 0.0,  0.375, 0.0, 12.0, 0.0,
    0.625, 0.0, -12.0, 0.0,  1.0, 0.0, 0.0, 0.0,

    0.0, 0.375, 0.0, 12.0,  0.375, 0.375, 12.0, 12.0,
    0.625, 0.375, -12.0, 12.0,  1.0, 0.375, 0.0, 12.0,

    0.0, 0.625, 0.0, -12.0,  0.375, 0.625, 12.0, -12.0,
    0.625, 0.625, -12.0, -12.0,  1.0, 0.625, 0.0, -12.0,

    0.0, 1.0, 0.0, 0.0,  0.375, 1.0, 12.0, 0.0,
    0.625, 1.0, -12.0, 0.0,  1.0, 1.0, 0.0, 0.0,
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
    0.0, 0.0, 0.0, 0.0,  0.5, 0.0, 64.0, 0.0,
    0.75, 0.0, -32.0, 0.0,  1.0, 0.0, 0.0, 0.0,

    0.0, 0.25, 0.0, 32.0,  0.5, 0.25, 64.0, 32.0,
    0.75, 0.25, -32.0, 32.0,  1.0, 0.25, 0.0, 32.0,

    0.0, 0.5, 0.0, -64.0,  0.5, 0.5, 64.0, -64.0,
    0.75, 0.5, -32.0, -64.0,  1.0, 0.5, 0.0, -64.0,

    0.0, 1.0, 0.0, 0.0,  0.5, 1.0, 64.0, 0.0,
    0.75, 1.0, -32.0, 0.0,  1.0, 1.0, 0.0, 0.0,
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

#[cfg_attr(rustfmt, rustfmt_skip)]
const SELECTION_BOX_DATA: &[f32] = &[
    0.0, 0.0, 0.0, 0.0,  0.25, 0.0, 8.0, 0.0,
    0.75, 0.0, -8.0, 0.0,  1.0, 0.0, 0.0, 0.0,

    0.0, 0.25, 0.0, 8.0,  0.25, 0.25, 8.0, 8.0,
    0.75, 0.25, -8.0, 8.0,  1.0, 0.25, 0.0, 8.0,

    0.0, 0.75, 0.0, -8.0,  0.25, 0.75, 8.0, -8.0,
    0.75, 0.75, -8.0, -8.0,  1.0, 0.75, 0.0, -8.0,

    0.0, 1.0, 0.0, 0.0,  0.25, 1.0, 8.0, 0.0,
    0.75, 1.0, -8.0, 0.0,  1.0, 1.0, 0.0, 0.0,
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const TRAY_DATA_1: &[f32] = &[
    0.0, 0.0, 0.0, 0.0,  0.5, 0.0, 64.0, 0.0,
    0.75, 0.0, -32.0, 0.0,  1.0, 0.0, 0.0, 0.0,

    0.0, 0.375, 0.0, 48.0,  0.5, 0.375, 64.0, 48.0,
    0.75, 0.375, -32.0, 48.0,  1.0, 0.375, 0.0, 48.0,

    0.0, 0.375, 0.0, -48.0,  0.5, 0.375, 64.0, -48.0,
    0.75, 0.375, -32.0, -48.0,  1.0, 0.375, 0.0, -48.0,

    0.0, 0.75, 0.0, 0.0,  0.5, 0.75, 64.0, 0.0,
    0.75, 0.75, -32.0, 0.0,  1.0, 0.75, 0.0, 0.0,
];
#[cfg_attr(rustfmt, rustfmt_skip)]
const TRAY_DATA_2: &[f32] = &[
    0.0, 0.75, 0.0, 0.0,  0.5, 0.75, 64.0, 0.0,
    0.75, 0.75, -32.0, 0.0,  1.0, 0.75, 0.0, 0.0,

    0.0, 0.875, 0.0, 16.0,  0.5, 0.875, 64.0, 16.0,
    0.75, 0.875, -32.0, 16.0,  1.0, 0.875, 0.0, 16.0,

    0.0, 0.875, 0.0, -16.0,  0.5, 0.875, 64.0, -16.0,
    0.75, 0.875, -32.0, -16.0,  1.0, 0.875, 0.0, -16.0,

    0.0, 1.0, 0.0, 0.0,  0.5, 1.0, 64.0, 0.0,
    0.75, 1.0, -32.0, 0.0,  1.0, 1.0, 0.0, 0.0,
];
const TRAY_TAB_WIDTH: f32 = 20.0;
const TRAY_TAB_UPPER_MARGIN: f32 = 34.0;
const TRAY_TAB_INNER_MARGIN: f32 = 16.0;
const TRAY_TAB_LOWER_MARGIN: f32 = 30.0;

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
    sampler: ShaderSampler<Texture2D>,
    ibuffer: IndexBuffer<u8>,
    _corners_vbuffer: VertexBuffer<u8>,
    _box_vbuffer: VertexBuffer<f32>,
    box_varray: VertexArray,
    _bubble_vbuffer: VertexBuffer<f32>,
    bubble_varray: VertexArray,
    _checkbox_vbuffer: VertexBuffer<f32>,
    checkbox_varray: VertexArray,
    _dialog_vbuffer: VertexBuffer<f32>,
    dialog_varray: VertexArray,
    _scroll_vbuffer: VertexBuffer<f32>,
    scroll_varray: VertexArray,
    _selection_box_vbuffer: VertexBuffer<f32>,
    selection_box_varray: VertexArray,
    _tray_vbuffer_1: VertexBuffer<f32>,
    tray_varray_1: VertexArray,
    _tray_vbuffer_2: VertexBuffer<f32>,
    tray_varray_2: VertexArray,
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
        let sampler = program.get_sampler(0, "Texture")?;
        let ibuffer = IndexBuffer::new(INDEX_DATA);
        let corners_vbuffer = VertexBuffer::new(CORNERS_DATA);

        let (box_varray, box_vbuffer) =
            make_vertices(&corners_vbuffer, BOX_DATA);
        let (bubble_varray, bubble_vbuffer) =
            make_vertices(&corners_vbuffer, BUBBLE_DATA);
        let (checkbox_varray, checkbox_vbuffer) =
            make_vertices(&corners_vbuffer, CHECKBOX_DATA);
        let (dialog_varray, dialog_vbuffer) =
            make_vertices(&corners_vbuffer, DIALOG_DATA);
        let (scroll_varray, scroll_vbuffer) =
            make_vertices(&corners_vbuffer, SCROLL_DATA);
        let (selection_box_varray, selection_box_vbuffer) =
            make_vertices(&corners_vbuffer, SELECTION_BOX_DATA);
        let (tray_varray_1, tray_vbuffer_1) =
            make_vertices(&corners_vbuffer, TRAY_DATA_1);
        let (tray_varray_2, tray_vbuffer_2) =
            make_vertices(&corners_vbuffer, TRAY_DATA_2);

        Ok(UiShader {
            program,
            texture,
            mvp,
            screen_rect,
            tex_rect,
            color1,
            color2,
            color3,
            sampler,
            ibuffer,
            _corners_vbuffer: corners_vbuffer,
            _box_vbuffer: box_vbuffer,
            box_varray,
            _bubble_vbuffer: bubble_vbuffer,
            bubble_varray,
            _checkbox_vbuffer: checkbox_vbuffer,
            checkbox_varray,
            _dialog_vbuffer: dialog_vbuffer,
            dialog_varray,
            _scroll_vbuffer: scroll_vbuffer,
            scroll_varray,
            _selection_box_vbuffer: selection_box_vbuffer,
            selection_box_varray,
            _tray_vbuffer_1: tray_vbuffer_1,
            tray_varray_1,
            _tray_vbuffer_2: tray_vbuffer_2,
            tray_varray_2,
        })
    }

    fn bind(
        &self,
        matrix: &Matrix4<f32>,
        screen_rect: &Rect<f32>,
        color1: &Color4,
        color2: &Color4,
        color3: &Color4,
        tex_rect: &Rect<f32>,
    ) {
        self.program.bind();
        self.mvp.set(matrix);
        self.screen_rect.set(screen_rect);
        self.tex_rect.set(tex_rect);
        self.color1.set(color1);
        self.color2.set(color2);
        self.color3.set(color3);
        self.sampler.set(&self.texture);
    }

    pub fn draw_box2(
        &self,
        matrix: &Matrix4<f32>,
        rect: &Rect<f32>,
        color1: &Color4,
        color2: &Color4,
        color3: &Color4,
    ) {
        let tex_rect = Rect::new(0.5, 0.125, 0.25, 0.125);
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.box_varray.bind();
        self.box_varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_box4(
        &self,
        matrix: &Matrix4<f32>,
        rect: &Rect<f32>,
        color1: &Color4,
        color2: &Color4,
        color3: &Color4,
    ) {
        let tex_rect = Rect::new(0.5, 0.0, 0.25, 0.125);
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.box_varray.bind();
        self.box_varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_bubble(
        &self,
        matrix: &Matrix4<f32>,
        rect: &Rect<f32>,
        color1: &Color4,
        color2: &Color4,
        color3: &Color4,
    ) {
        let tex_rect = Rect::new(0.75, 0.375, 0.125, 0.125);
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.bubble_varray.bind();
        self.bubble_varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_checkbox(
        &self,
        matrix: &Matrix4<f32>,
        rect: &Rect<f32>,
        color1: &Color4,
        color2: &Color4,
        color3: &Color4,
        checked: bool,
    ) {
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

    pub fn draw_dialog(
        &self,
        matrix: &Matrix4<f32>,
        rect: &Rect<f32>,
        color1: &Color4,
        color2: &Color4,
        color3: &Color4,
    ) {
        let tex_rect = Rect::new(0.0, 0.5, 0.5, 0.5);
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.dialog_varray.bind();
        self.dialog_varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_icon(
        &self,
        matrix: &Matrix4<f32>,
        rect: &Rect<f32>,
        icon_index: usize,
        color1: &Color4,
        color2: &Color4,
        color3: &Color4,
    ) {
        let (icon_row, icon_col) = div_mod_floor(icon_index, 3);
        let tex_rect = Rect::new(
            0.5 + 0.125 * (icon_col as f32),
            0.75 + 0.125 * (icon_row as f32),
            0.125,
            0.125,
        );
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.checkbox_varray.bind();
        self.checkbox_varray
            .draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_list_frame(
        &self,
        matrix: &Matrix4<f32>,
        rect: &Rect<f32>,
        color1: &Color4,
        color2: &Color4,
        color3: &Color4,
    ) {
        let tex_rect = Rect::new(0.75, 0.25, 0.125, 0.125);
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.scroll_varray.bind();
        self.scroll_varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_list_item(
        &self,
        matrix: &Matrix4<f32>,
        rect: &Rect<f32>,
        color1: &Color4,
        color2: &Color4,
        color3: &Color4,
    ) {
        let tex_rect = Rect::new(0.875, 0.25, 0.125, 0.125);
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.scroll_varray.bind();
        self.scroll_varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_scroll_bar(
        &self,
        matrix: &Matrix4<f32>,
        rect: &Rect<f32>,
        color1: &Color4,
        color2: &Color4,
        color3: &Color4,
    ) {
        let tex_rect = Rect::new(0.875, 0.0, 0.125, 0.125);
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.scroll_varray.bind();
        self.scroll_varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_scroll_handle(
        &self,
        matrix: &Matrix4<f32>,
        rect: &Rect<f32>,
        color1: &Color4,
        color2: &Color4,
        color3: &Color4,
    ) {
        let tex_rect = Rect::new(0.75, 0.0, 0.125, 0.125);
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.scroll_varray.bind();
        self.scroll_varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_selection_box(
        &self,
        matrix: &Matrix4<f32>,
        rect: &Rect<f32>,
        color1: &Color4,
        color2: &Color4,
        color3: &Color4,
    ) {
        let tex_rect = Rect::new(0.875, 0.375, 0.125, 0.125);
        self.bind(matrix, rect, color1, color2, color3, &tex_rect);
        self.selection_box_varray.bind();
        self.selection_box_varray
            .draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn draw_tray(
        &self,
        matrix: &Matrix4<f32>,
        rect: &Rect<f32>,
        tab_size: f32,
        flip_horz: bool,
        color1: &Color4,
        color2: &Color4,
        color3: &Color4,
    ) {
        let tex_rect = Rect::new(0.0, 0.0, 0.5, 0.5);
        let matrix = if flip_horz {
            matrix
                * Matrix4::trans2(rect.x + rect.width, rect.y)
                * Matrix4::scale2(-1.0, 1.0)
        } else {
            matrix * Matrix4::trans2(rect.x, rect.y)
        };

        let rect1 = Rect::new(
            0.0,
            0.0,
            rect.width + TRAY_TAB_WIDTH,
            TRAY_TAB_UPPER_MARGIN
                + 2.0 * TRAY_TAB_INNER_MARGIN
                + TRAY_TAB_LOWER_MARGIN
                + tab_size,
        );
        self.bind(&matrix, &rect1, color1, color2, color3, &tex_rect);
        self.tray_varray_1.bind();
        self.tray_varray_1.draw_elements(Primitive::Triangles, &self.ibuffer);

        let rect2 = Rect::new(
            0.0,
            rect1.height,
            rect1.width,
            rect.height - rect1.height,
        );
        self.bind(&matrix, &rect2, color1, color2, color3, &tex_rect);
        self.tray_varray_2.bind();
        self.tray_varray_2.draw_elements(Primitive::Triangles, &self.ibuffer);
    }

    pub fn tray_tab_rect(
        rect: Rect<f32>,
        tab_size: f32,
        flip_horz: bool,
    ) -> Rect<f32> {
        let left =
            if flip_horz { rect.x - TRAY_TAB_WIDTH } else { rect.right() };
        Rect::new(
            left,
            rect.y + TRAY_TAB_UPPER_MARGIN,
            TRAY_TAB_WIDTH,
            tab_size + 2.0 * TRAY_TAB_INNER_MARGIN,
        )
    }
}

fn make_vertices(
    corners: &VertexBuffer<u8>,
    data: &[f32],
) -> (VertexArray, VertexBuffer<f32>) {
    let varray = VertexArray::new(3);
    let vbuffer = VertexBuffer::new(data);
    varray.bind();
    corners.attribi(0, 2, 0, 0);
    vbuffer.attribf(1, 2, 4, 0);
    vbuffer.attribf(2, 2, 4, 2);
    (varray, vbuffer)
}

//===========================================================================//
