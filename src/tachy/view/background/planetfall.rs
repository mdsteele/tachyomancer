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

use super::shared::BackgroundView;
use cgmath::{self, Deg, Matrix4, Point3, SquareMatrix, Vector3, vec3};
use tachy::geom::{Color3, RectSize};
use tachy::gl::{Depth, HeightmapModel, Model, ModelBuilder};
use tachy::gui::{Event, Resources, Ui};

//===========================================================================//

pub struct PlanetfallBackgroundView {
    p_matrix: Matrix4<f32>,
    terrain_model: HeightmapModel,
    terrain_rotation: Deg<f32>,
    sky_model: Model,
}

impl PlanetfallBackgroundView {
    pub fn new(screen_size: RectSize<f32>) -> PlanetfallBackgroundView {
        let aspect = screen_size.width / screen_size.height;
        let p_matrix = cgmath::perspective(Deg(45.0), aspect, 0.1, 1000.0);

        let mut sky = ModelBuilder::new();
        sky.plane(Point3::new(0.0, 0.0, -100.0),
                  RectSize::new(300.0, 300.0),
                  Vector3::unit_z(),
                  Color3::WHITE);

        PlanetfallBackgroundView {
            p_matrix,
            terrain_model: HeightmapModel::new(64),
            terrain_rotation: Deg(0.0),
            sky_model: sky.build(),
        }
    }
}

impl BackgroundView for PlanetfallBackgroundView {
    fn draw(&self, resources: &Resources) {
        let _depth = Depth::new();
        let v_matrix = Matrix4::look_at(Point3::new(0.0, 0.0, 100.0),
                                        Point3::new(0.0, 0.0, 0.0),
                                        Vector3::unit_y());
        let light_dir_world_space = Vector3::new(-3.0, 3.0, 10.0);

        let m_matrix = Matrix4::from_angle_y(self.terrain_rotation) *
            Matrix4::from_translation(vec3(0.0, -20.0, 0.0)) *
            Matrix4::from_nonuniform_scale(100.0, 30.0, 100.0) *
            Matrix4::from_translation(vec3(-0.5, -0.5, -0.5));
        resources.shaders().heightmap().render(&self.p_matrix,
                                               &v_matrix,
                                               light_dir_world_space,
                                               &m_matrix,
                                               resources
                                                   .textures()
                                                   .valley_heightmap(),
                                               resources
                                                   .textures()
                                                   .red_planet(),
                                               &self.terrain_model);

        resources.shaders().scene().render(&self.p_matrix,
                                           &v_matrix,
                                           light_dir_world_space,
                                           &Matrix4::identity(),
                                           resources.textures().starfield(),
                                           &self.sky_model);
    }

    fn on_event(&mut self, event: &Event, ui: &mut Ui) {
        match event {
            Event::ClockTick(tick) => {
                self.terrain_rotation += Deg(15.0) * (tick.elapsed as f32);
                ui.request_redraw();
            }
            _ => {}
        }
    }
}

//===========================================================================//
