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

mod chip;
mod frame;
mod port;
mod portrait;
mod scene;
mod solid;
mod ui;

use self::chip::ChipShader;
pub use self::frame::FrameBufferShader;
pub use self::port::PortShader;
pub use self::portrait::PortraitShader;
pub use self::scene::SceneShader;
pub use self::solid::SolidShader;
pub use self::ui::UiShader;
use cgmath::Matrix4;
use tachy::geom::{Color3, Color4, MatrixExt, Rect};
use tachy::gl::{Primitive, Shader, ShaderProgram, ShaderType, ShaderUniform,
                VertexArray, VertexBuffer};

//===========================================================================//

const BOARD_VERT_CODE: &[u8] = include_bytes!("board.vert");
const BOARD_FRAG_CODE: &[u8] = include_bytes!("board.frag");

const ICON_VERT_CODE: &[u8] = include_bytes!("icon.vert");
const ICON_FRAG_CODE: &[u8] = include_bytes!("icon.frag");

const WIRE_VERT_CODE: &[u8] = include_bytes!("wire.vert");
const WIRE_FRAG_CODE: &[u8] = include_bytes!("wire.frag");

//===========================================================================//

pub struct Shaders {
    board: BoardShader,
    chip: ChipShader,
    frame: FrameBufferShader,
    icon: IconShader,
    port: PortShader,
    portrait: PortraitShader,
    scene: SceneShader,
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

        let chip = ChipShader::new()?;
        let frame = FrameBufferShader::new()?;
        let icon = IconShader::new()?;
        let port = PortShader::new()?;
        let portrait = PortraitShader::new()?;
        let scene = SceneShader::new()?;
        let solid = SolidShader::new()?;
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
            frame,
            icon,
            port,
            portrait,
            scene,
            solid,
            ui,
            wire,
        };
        Ok(shaders)
    }

    pub fn board(&self) -> &BoardShader { &self.board }

    pub fn chip(&self) -> &ChipShader { &self.chip }

    pub fn frame(&self) -> &FrameBufferShader { &self.frame }

    pub fn icon(&self) -> &IconShader { &self.icon }

    pub fn port(&self) -> &PortShader { &self.port }

    pub fn portrait(&self) -> &PortraitShader { &self.portrait }

    pub fn scene(&self) -> &SceneShader { &self.scene }

    pub fn solid(&self) -> &SolidShader { &self.solid }

    pub fn ui(&self) -> &UiShader { &self.ui }

    pub fn wire(&self) -> &WireShader { &self.wire }
}

//===========================================================================//
// TODO: Move these to separate modules.

pub struct BoardShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    coords_rect: ShaderUniform<Rect<f32>>,
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

    pub fn draw(&self, matrix: &Matrix4<f32>, coords_rect: Rect<f32>) {
        self.program.bind();
        self.mvp.set(matrix);
        self.coords_rect.set(&coords_rect);
        self.varray.bind();
        self.varray.draw(Primitive::TriangleStrip, 0, 4);
    }
}

//===========================================================================//

pub struct IconShader {
    program: ShaderProgram,
    color: ShaderUniform<Color4>,
    icon_index: ShaderUniform<u32>,
    mvp: ShaderUniform<Matrix4<f32>>,
    varray: VertexArray,
    rect_vbuffer: VertexBuffer<u8>,
}

impl IconShader {
    fn new() -> Result<IconShader, String> {
        let vert =
            Shader::new(ShaderType::Vertex, "icon.vert", ICON_VERT_CODE)?;
        let frag =
            Shader::new(ShaderType::Fragment, "icon.frag", ICON_FRAG_CODE)?;
        let program = ShaderProgram::new(&[&vert, &frag])?;
        let color = program.get_uniform("IconColor")?;
        let icon_index = program.get_uniform("IconIndex")?;
        let mvp = program.get_uniform("MVP")?;
        let varray = VertexArray::new(1);
        let rect_vbuffer = VertexBuffer::new(&[0, 0, 1, 0, 0, 1, 1, 1]);
        Ok(IconShader {
               program,
               color,
               icon_index,
               mvp,
               varray,
               rect_vbuffer,
           })
    }

    pub fn draw(&self, matrix: &Matrix4<f32>, rect: Rect<f32>, index: u32,
                color: &Color4) {
        self.program.bind();
        self.color.set(color);
        self.icon_index.set(&index);
        let mvp = matrix * Matrix4::trans2(rect.x, rect.y) *
            Matrix4::scale2(rect.width, rect.height);
        self.mvp.set(&mvp);
        self.varray.bind();
        self.rect_vbuffer.attribi(0, 2, 0, 0);
        self.varray.draw(Primitive::TriangleStrip, 0, 4);
    }
}

//===========================================================================//

pub struct WireShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    wire_color: ShaderUniform<Color3>,
    hilight_color: ShaderUniform<Color4>,
}

impl WireShader {
    fn new(program: ShaderProgram) -> Result<WireShader, String> {
        let mvp = program.get_uniform("MVP")?;
        let wire_color = program.get_uniform("WireColor")?;
        let hilight_color = program.get_uniform("HilightColor")?;
        Ok(WireShader {
               program,
               mvp,
               wire_color,
               hilight_color,
           })
    }

    pub fn bind(&self) { self.program.bind(); }

    pub fn set_wire_color(&self, color: &Color3) {
        self.wire_color.set(color);
    }

    pub fn set_hilight_color(&self, color: &Color4) {
        self.hilight_color.set(color);
    }

    pub fn set_mvp(&self, mvp: &Matrix4<f32>) { self.mvp.set(mvp); }
}

//===========================================================================//
