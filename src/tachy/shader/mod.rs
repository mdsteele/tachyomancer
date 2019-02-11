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

use cgmath::{Matrix4, Vector2, Vector3, Vector4};
use tachy::geom::{Color4, MatrixExt, Rect};
use tachy::gl::{IndexBuffer, Primitive, Shader, ShaderProgram, ShaderType,
                ShaderUniform, Texture2D, VertexArray, VertexBuffer};

//===========================================================================//

const BOARD_VERT_CODE: &[u8] = include_bytes!("board.vert");
const BOARD_FRAG_CODE: &[u8] = include_bytes!("board.frag");

const CHIP_VERT_CODE: &[u8] = include_bytes!("chip.vert");
const CHIP_FRAG_CODE: &[u8] = include_bytes!("chip.frag");

const SOLID_VERT_CODE: &[u8] = include_bytes!("solid.vert");
const SOLID_FRAG_CODE: &[u8] = include_bytes!("solid.frag");

const UI_VERT_CODE: &[u8] = include_bytes!("ui.vert");
const UI_FRAG_CODE: &[u8] = include_bytes!("ui.frag");
#[cfg_attr(rustfmt, rustfmt_skip)]
const UI_TEXTURE_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/texture/ui.png"));

const WIRE_VERT_CODE: &[u8] = include_bytes!("wire.vert");
const WIRE_FRAG_CODE: &[u8] = include_bytes!("wire.frag");

//===========================================================================//

pub struct Shaders {
    board: BoardShader,
    chip: ChipShader,
    solid: SolidShader,
    ui: UiShader,
    wire: WireShader,
}

impl Shaders {
    pub fn new() -> Result<Shaders, String> {
        let board_vert =
            Shader::new(ShaderType::Vertex, "board.vert", BOARD_VERT_CODE)?;
        let board_frag =
            Shader::new(ShaderType::Fragment, "board.frag", BOARD_FRAG_CODE)?;
        let board_prog = ShaderProgram::new(&[&board_vert, &board_frag])?;
        let board = BoardShader::new(board_prog)?;

        let chip_vert =
            Shader::new(ShaderType::Vertex, "chip.vert", CHIP_VERT_CODE)?;
        let chip_frag =
            Shader::new(ShaderType::Fragment, "chip.frag", CHIP_FRAG_CODE)?;
        let chip_prog = ShaderProgram::new(&[&chip_vert, &chip_frag])?;
        let chip = ChipShader::new(chip_prog)?;

        let solid_vert =
            Shader::new(ShaderType::Vertex, "solid.vert", SOLID_VERT_CODE)?;
        let solid_frag =
            Shader::new(ShaderType::Fragment, "solid.frag", SOLID_FRAG_CODE)?;
        let solid_prog = ShaderProgram::new(&[&solid_vert, &solid_frag])?;
        let solid = SolidShader::new(solid_prog)?;

        let ui = UiShader::new()?;

        let wire_vert =
            Shader::new(ShaderType::Vertex, "wire.vert", WIRE_VERT_CODE)?;
        let wire_frag =
            Shader::new(ShaderType::Fragment, "wire.frag", WIRE_FRAG_CODE)?;
        let wire_prog = ShaderProgram::new(&[&wire_vert, &wire_frag])?;
        let wire = WireShader::new(wire_prog)?;

        let shaders = Shaders {
            board,
            chip,
            solid,
            ui,
            wire,
        };
        Ok(shaders)
    }

    pub fn board(&self) -> &BoardShader { &self.board }

    pub fn chip(&self) -> &ChipShader { &self.chip }

    pub fn solid(&self) -> &SolidShader { &self.solid }

    pub fn ui(&self) -> &UiShader { &self.ui }

    pub fn wire(&self) -> &WireShader { &self.wire }
}

//===========================================================================//

pub struct BoardShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    coords_rect: ShaderUniform<Vector4<f32>>,
    varray: VertexArray,
    _vbuffer: VertexBuffer<u8>,
}

impl BoardShader {
    fn new(program: ShaderProgram) -> Result<BoardShader, String> {
        let mvp = program.get_uniform("MVP")?;
        let coords_rect = program.get_uniform("CoordsRect")?;
        let varray = VertexArray::new(1);
        let vbuffer = VertexBuffer::new(&[0, 0, 1, 0, 0, 1, 1, 1]);
        varray.bind();
        vbuffer.attribf(0, 2, 0, 0);
        Ok(BoardShader {
               program,
               mvp,
               coords_rect,
               varray,
               _vbuffer: vbuffer,
           })
    }

    pub fn draw(&self, matrix: &Matrix4<f32>, coords_rect: Vector4<f32>) {
        self.program.bind();
        self.mvp.set(matrix);
        self.coords_rect.set(&coords_rect);
        self.varray.bind();
        self.varray.draw(Primitive::TriangleStrip, 0, 4);
    }
}

//===========================================================================//

pub struct ChipShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    icon_coords: ShaderUniform<Vector2<u32>>,
    varray: VertexArray,
    _vbuffer: VertexBuffer<u8>,
}

impl ChipShader {
    fn new(program: ShaderProgram) -> Result<ChipShader, String> {
        let mvp = program.get_uniform("MVP")?;
        let icon_coords = program.get_uniform("IconCoords")?;
        let varray = VertexArray::new(1);
        let vbuffer = VertexBuffer::new(&[0, 0, 1, 0, 0, 1, 1, 1]);
        varray.bind();
        vbuffer.attribi(0, 2, 0, 0);
        Ok(ChipShader {
               program,
               mvp,
               icon_coords,
               varray,
               _vbuffer: vbuffer,
           })
    }

    pub fn draw(&self, matrix: &Matrix4<f32>, icon_coords: Vector2<u32>) {
        self.program.bind();
        self.mvp.set(matrix);
        self.icon_coords.set(&icon_coords);
        self.varray.bind();
        self.varray.draw(Primitive::TriangleStrip, 0, 4);
    }
}

//===========================================================================//

pub struct SolidShader {
    program: ShaderProgram,
    color: ShaderUniform<Vector3<f32>>,
    mvp: ShaderUniform<Matrix4<f32>>,
    varray: VertexArray,
    rect_vbuffer: VertexBuffer<u8>,
}

impl SolidShader {
    fn new(program: ShaderProgram) -> Result<SolidShader, String> {
        let color = program.get_uniform("SolidColor")?;
        let mvp = program.get_uniform("MVP")?;
        let varray = VertexArray::new(1);
        let rect_vbuffer =
            VertexBuffer::new(&[0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 0]);
        Ok(SolidShader {
               program,
               color,
               mvp,
               varray,
               rect_vbuffer,
           })
    }

    pub fn fill_rect(&self, matrix: &Matrix4<f32>, color: (f32, f32, f32),
                     rect: Rect<f32>) {
        self.program.bind();
        self.color.set(&color.into());
        let mvp = matrix * Matrix4::trans2(rect.x, rect.y) *
            Matrix4::scale2(rect.width, rect.height);
        self.mvp.set(&mvp);
        self.varray.bind();
        self.rect_vbuffer.attribf(0, 3, 0, 0);
        self.varray.draw(Primitive::TriangleStrip, 0, 4);
    }
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
const UI_INDEX_DATA: &[u8] = &[
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
const UI_CORNERS_DATA: &[u8] = &[
    0, 0,  0, 0,  1, 0,  1, 0,
    0, 0,  0, 0,  1, 0,  1, 0,
    0, 1,  0, 1,  1, 1,  1, 1,
    0, 1,  0, 1,  1, 1,  1, 1,
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const UI_DIALOG_DATA: &[f32] = &[
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
const UI_RECT4_DATA: &[f32] = &[
    0.5, 0.0, 0.0, 0.0,  0.575, 0.0, 19.2, 0.0,
    0.675, 0.0, -19.2, 0.0,  0.75, 0.0, 0.0, 0.0,

    0.5, 0.075, 0.0, 19.2,  0.575, 0.075, 19.2, 19.2,
    0.675, 0.075, -19.2, 19.2,  0.75, 0.075, 0.0, 19.2,

    0.5, 0.175, 0.0, -19.2,  0.575, 0.175, 19.2, -19.2,
    0.675, 0.175, -19.2, -19.2,  0.75, 0.175, 0.0, -19.2,

    0.5, 0.25, 0.0, 0.0,  0.575, 0.25, 19.2, 0.0,
    0.675, 0.25, -19.2, 0.0,  0.75, 0.25, 0.0, 0.0,
];

pub struct UiShader {
    program: ShaderProgram,
    texture: Texture2D,
    mvp: ShaderUniform<Matrix4<f32>>,
    rect: ShaderUniform<Rect<f32>>,
    color1: ShaderUniform<Color4>,
    color2: ShaderUniform<Color4>,
    color3: ShaderUniform<Color4>,
    ibuffer: IndexBuffer<u8>,
    _corners_vbuffer: VertexBuffer<u8>,
    _dialog_vbuffer: VertexBuffer<f32>,
    dialog_varray: VertexArray,
    _rect4_vbuffer: VertexBuffer<f32>,
    rect4_varray: VertexArray,
}

impl UiShader {
    fn new() -> Result<UiShader, String> {
        let vert = Shader::new(ShaderType::Vertex, "ui.vert", UI_VERT_CODE)?;
        let frag = Shader::new(ShaderType::Fragment, "ui.frag", UI_FRAG_CODE)?;
        let program = ShaderProgram::new(&[&vert, &frag])?;
        let texture = Texture2D::from_png("texture/ui", UI_TEXTURE_PNG_DATA)?;
        let mvp = program.get_uniform("MVP")?;
        let rect = program.get_uniform("Rect")?;
        let color1 = program.get_uniform("Color1")?;
        let color2 = program.get_uniform("Color2")?;
        let color3 = program.get_uniform("Color3")?;
        let ibuffer = IndexBuffer::new(UI_INDEX_DATA);
        let corners_vbuffer = VertexBuffer::new(UI_CORNERS_DATA);

        let dialog_varray = VertexArray::new(3);
        let dialog_vbuffer = VertexBuffer::new(UI_DIALOG_DATA);
        dialog_varray.bind();
        corners_vbuffer.attribi(0, 2, 0, 0);
        dialog_vbuffer.attribf(1, 2, 4, 0);
        dialog_vbuffer.attribf(2, 2, 4, 2);

        let rect4_varray = VertexArray::new(3);
        let rect4_vbuffer = VertexBuffer::new(UI_RECT4_DATA);
        rect4_varray.bind();
        corners_vbuffer.attribi(0, 2, 0, 0);
        rect4_vbuffer.attribf(1, 2, 4, 0);
        rect4_vbuffer.attribf(2, 2, 4, 2);

        let shader = UiShader {
            program,
            texture,
            mvp,
            rect,
            color1,
            color2,
            color3,
            ibuffer,
            _corners_vbuffer: corners_vbuffer,
            _dialog_vbuffer: dialog_vbuffer,
            dialog_varray,
            _rect4_vbuffer: rect4_vbuffer,
            rect4_varray,
        };
        Ok(shader)
    }

    pub fn draw_dialog(&self, matrix: &Matrix4<f32>, rect: &Rect<f32>,
                       color1: &Color4, color2: &Color4, color3: &Color4) {
        self.program.bind();
        self.texture.bind();
        self.mvp.set(matrix);
        self.rect.set(rect);
        self.color1.set(color1);
        self.color2.set(color2);
        self.color3.set(color3);
        self.dialog_varray.bind();
        self.dialog_varray.draw_elements(Primitive::Triangles,
                                         &self.ibuffer,
                                         self.ibuffer.len());
    }

    pub fn draw_rect4(&self, matrix: &Matrix4<f32>, rect: &Rect<f32>,
                      color1: &Color4, color2: &Color4, color3: &Color4) {
        self.program.bind();
        self.texture.bind();
        self.mvp.set(matrix);
        self.rect.set(rect);
        self.color1.set(color1);
        self.color2.set(color2);
        self.color3.set(color3);
        self.rect4_varray.bind();
        self.rect4_varray.draw_elements(Primitive::Triangles,
                                        &self.ibuffer,
                                        self.ibuffer.len());
    }
}

//===========================================================================//

pub struct WireShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    wire_color: ShaderUniform<Color4>,
}

impl WireShader {
    fn new(program: ShaderProgram) -> Result<WireShader, String> {
        let mvp = program.get_uniform("MVP")?;
        let wire_color = program.get_uniform("WireColor")?;
        Ok(WireShader {
               program,
               mvp,
               wire_color,
           })
    }

    pub fn bind(&self) { self.program.bind(); }

    pub fn set_wire_color(&self, color: &Color4) {
        self.wire_color.set(color);
    }

    pub fn set_mvp(&self, mvp: &Matrix4<f32>) { self.mvp.set(mvp); }
}

//===========================================================================//
