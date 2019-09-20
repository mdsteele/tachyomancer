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

use crate::tachy::gl::{
    Model, Shader, ShaderProgram, ShaderSampler, ShaderType, ShaderUniform,
    Texture2D,
};
use cgmath::{InnerSpace, Matrix4, Vector3};

//===========================================================================//

const SCENE_VERT_CODE: &[u8] = include_bytes!("scene.vert");
const SCENE_FRAG_CODE: &[u8] = include_bytes!("scene.frag");

//===========================================================================//

pub struct SceneShader {
    program: ShaderProgram,
    mv: ShaderUniform<Matrix4<f32>>,
    p: ShaderUniform<Matrix4<f32>>,
    ambient_light: ShaderUniform<f32>,
    diffuse_light: ShaderUniform<f32>,
    light_dir_cam_space: ShaderUniform<Vector3<f32>>,
    texture: ShaderSampler<Texture2D>,
}

impl SceneShader {
    pub(super) fn new() -> Result<SceneShader, String> {
        let vert =
            Shader::new(ShaderType::Vertex, "scene.vert", SCENE_VERT_CODE)?;
        let frag =
            Shader::new(ShaderType::Fragment, "scene.frag", SCENE_FRAG_CODE)?;
        let program = ShaderProgram::new(&[&vert, &frag])?;

        let mv = program.get_uniform("MV")?;
        let p = program.get_uniform("P")?;
        let ambient_light = program.get_uniform("AmbientLight")?;
        let diffuse_light = program.get_uniform("DiffuseLight")?;
        let light_dir_cam_space = program.get_uniform("LightDirCamSpace")?;
        let texture = program.get_sampler(0, "Texture")?;

        Ok(SceneShader {
            program,
            mv,
            p,
            ambient_light,
            diffuse_light,
            light_dir_cam_space,
            texture,
        })
    }

    pub fn render(
        &self,
        p_matrix: &Matrix4<f32>,
        v_matrix: &Matrix4<f32>,
        light_dir_world_space: Vector3<f32>,
        m_matrix: &Matrix4<f32>,
        texture: &Texture2D,
        model: &Model,
    ) {
        self.program.bind();
        self.p.set(p_matrix);
        self.mv.set(&(v_matrix * m_matrix));
        // TODO: Make light levels a method parameter
        self.ambient_light.set(&0.3);
        self.diffuse_light.set(&0.7);
        let light_dir_cam_space = (v_matrix
            * light_dir_world_space.extend(0.0))
        .truncate()
        .normalize();
        self.light_dir_cam_space.set(&light_dir_cam_space);
        self.texture.set(texture);
        model.draw();
    }
}

//===========================================================================//
