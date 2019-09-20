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

mod depth;
mod frame;
mod heightmap;
mod index;
mod model;
mod program;
mod sampler;
mod shader;
mod stencil;
mod texture;
mod uniform;
mod vertex;

pub use self::depth::Depth;
pub use self::frame::FrameBuffer;
pub use self::heightmap::HeightmapModel;
pub use self::index::IndexBuffer;
pub use self::model::{Model, ModelBuilder, ModelBuilderContext};
pub use self::program::ShaderProgram;
pub use self::sampler::ShaderSampler;
pub use self::shader::{Shader, ShaderType};
pub use self::stencil::Stencil;
pub use self::texture::{Texture1D, Texture2D, Texture2DMultisample};
pub use self::uniform::ShaderUniform;
pub use self::vertex::{Primitive, VertexArray, VertexBuffer};

//===========================================================================//
