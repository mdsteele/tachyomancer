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

mod board;
mod chip;
mod diagram;
mod frame;
mod heightmap;
mod icon;
mod port;
mod portrait;
mod scene;
mod shadow;
mod solid;
mod ui;
mod wire;

pub use self::board::BoardShader;
pub use self::chip::ChipShader;
pub use self::diagram::DiagramShader;
pub use self::frame::FrameBufferShader;
pub use self::heightmap::HeightmapShader;
pub use self::icon::IconShader;
pub use self::port::PortShader;
pub use self::portrait::PortraitShader;
pub use self::scene::SceneShader;
pub use self::shadow::ShadowShader;
pub use self::solid::SolidShader;
pub use self::ui::UiShader;
pub use self::wire::WireShader;
use tachy::geom::RectSize;

//===========================================================================//

pub struct Shaders {
    board: BoardShader,
    chip: ChipShader,
    diagram: DiagramShader,
    frame: FrameBufferShader,
    heightmap: HeightmapShader,
    icon: IconShader,
    port: PortShader,
    portrait: PortraitShader,
    scene: SceneShader,
    shadow: ShadowShader,
    solid: SolidShader,
    ui: UiShader,
    wire: WireShader,
}

impl Shaders {
    pub fn new(window_size: RectSize<i32>) -> Result<Shaders, String> {
        Ok(Shaders {
            board: BoardShader::new()?,
            chip: ChipShader::new()?,
            diagram: DiagramShader::new()?,
            frame: FrameBufferShader::new(window_size)?,
            heightmap: HeightmapShader::new()?,
            icon: IconShader::new()?,
            port: PortShader::new()?,
            portrait: PortraitShader::new()?,
            scene: SceneShader::new()?,
            shadow: ShadowShader::new()?,
            solid: SolidShader::new()?,
            ui: UiShader::new()?,
            wire: WireShader::new()?,
        })
    }

    pub fn board(&self) -> &BoardShader {
        &self.board
    }

    pub fn chip(&self) -> &ChipShader {
        &self.chip
    }

    pub fn diagram(&self) -> &DiagramShader {
        &self.diagram
    }

    pub fn frame(&self) -> &FrameBufferShader {
        &self.frame
    }

    pub fn heightmap(&self) -> &HeightmapShader {
        &self.heightmap
    }

    pub fn icon(&self) -> &IconShader {
        &self.icon
    }

    pub fn port(&self) -> &PortShader {
        &self.port
    }

    pub fn portrait(&self) -> &PortraitShader {
        &self.portrait
    }

    pub fn scene(&self) -> &SceneShader {
        &self.scene
    }

    pub fn shadow(&self) -> &ShadowShader {
        &self.shadow
    }

    pub fn solid(&self) -> &SolidShader {
        &self.solid
    }

    pub fn ui(&self) -> &UiShader {
        &self.ui
    }

    pub fn wire(&self) -> &WireShader {
        &self.wire
    }
}

//===========================================================================//
