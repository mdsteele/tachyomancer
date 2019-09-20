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

use crate::mancer::gl::{
    HeightmapModel, Shader, ShaderProgram, ShaderSampler, ShaderType,
    ShaderUniform, Texture2D,
};
use cgmath::{InnerSpace, Matrix4, Vector3};
use tachy::geom::Rect;

//===========================================================================//

const HEIGHTMAP_VERT_CODE: &[u8] = include_bytes!("heightmap.vert");
const HEIGHTMAP_FRAG_CODE: &[u8] = include_bytes!("heightmap.frag");

//===========================================================================//

pub struct HeightmapShader {
    program: ShaderProgram,
    mv: ShaderUniform<Matrix4<f32>>,
    p: ShaderUniform<Matrix4<f32>>,
    ambient_light: ShaderUniform<f32>,
    diffuse_light: ShaderUniform<f32>,
    light_dir_cam_space: ShaderUniform<Vector3<f32>>,
    heightmap: ShaderSampler<Texture2D>,
    heightmap_rect: ShaderUniform<Rect<f32>>,
    texture: ShaderSampler<Texture2D>,
    texture_rect: ShaderUniform<Rect<f32>>,
}

impl HeightmapShader {
    pub(super) fn new() -> Result<HeightmapShader, String> {
        let vert = Shader::new(
            ShaderType::Vertex,
            "heightmap.vert",
            HEIGHTMAP_VERT_CODE,
        )?;
        let frag = Shader::new(
            ShaderType::Fragment,
            "heightmap.frag",
            HEIGHTMAP_FRAG_CODE,
        )?;
        let program = ShaderProgram::new(&[&vert, &frag])?;

        let mv = program.get_uniform("MV")?;
        let p = program.get_uniform("P")?;
        let heightmap = program.get_sampler(1, "Heightmap")?;
        let heightmap_rect = program.get_uniform("HeightmapRect")?;
        let texture_rect = program.get_uniform("TextureRect")?;
        let ambient_light = program.get_uniform("AmbientLight")?;
        let diffuse_light = program.get_uniform("DiffuseLight")?;
        let light_dir_cam_space = program.get_uniform("LightDirCamSpace")?;
        let texture = program.get_sampler(0, "Texture")?;

        Ok(HeightmapShader {
            program,
            mv,
            p,
            ambient_light,
            diffuse_light,
            light_dir_cam_space,
            heightmap,
            heightmap_rect,
            texture,
            texture_rect,
        })
    }

    pub fn render(
        &self,
        p_matrix: &Matrix4<f32>,
        v_matrix: &Matrix4<f32>,
        light_dir_world_space: Vector3<f32>,
        m_matrix: &Matrix4<f32>,
        heightmap: &Texture2D,
        heightmap_rect: Rect<f32>,
        texture: &Texture2D,
        texture_rect: Rect<f32>,
        model: &HeightmapModel,
    ) {
        let light_dir_cam_space = (v_matrix
            * light_dir_world_space.extend(0.0))
        .truncate()
        .normalize();

        self.program.bind();
        self.mv.set(&(v_matrix * m_matrix));
        self.p.set(p_matrix);
        self.ambient_light.set(&0.3);
        self.diffuse_light.set(&0.7);
        self.light_dir_cam_space.set(&light_dir_cam_space);
        self.heightmap.set(heightmap);
        self.heightmap_rect.set(&heightmap_rect);
        self.texture.set(texture);
        self.texture_rect.set(&texture_rect);
        // TODO: Make light levels a method parameter
        model.draw();
    }
}

//===========================================================================//
