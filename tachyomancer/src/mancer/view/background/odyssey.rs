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
use crate::mancer::gl::{Depth, Model, ModelBuilder};
use crate::mancer::gui::{Event, Resources, Ui};
use cgmath::{self, vec3, Deg, Matrix4, Point3, SquareMatrix, Vector3};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tachy::geom::{Color3, RectSize};

//===========================================================================//

const PLANET_RADIUS: f32 = 50.0;
const PLANET_TILT_X: Deg<f32> = Deg(25.0);
const PLANET_TILT_Z: Deg<f32> = Deg(10.0);

//===========================================================================//

pub struct OdysseyBackgroundView {
    p_matrix: Matrix4<f32>,
    planet_model: Model,
    planet_rotation: i32,
    starfield_model: Model,
}

impl OdysseyBackgroundView {
    pub fn new(screen_size: RectSize<f32>) -> OdysseyBackgroundView {
        let aspect = screen_size.width / screen_size.height;
        let p_matrix = cgmath::perspective(Deg(45.0), aspect, 0.1, 1000.0);

        let mut planet = ModelBuilder::new();
        planet.sphere(
            Point3::new(0.0, 0.0, 0.0),
            PLANET_RADIUS,
            24,
            Color3::WHITE,
        );

        let mut starfield = ModelBuilder::new();
        starfield.plane(
            Point3::new(0.0, 0.0, -100.0),
            RectSize::new(300.0, 300.0),
            Vector3::unit_z(),
            Color3::WHITE,
        );

        OdysseyBackgroundView {
            p_matrix,
            planet_model: planet.build(),
            planet_rotation: get_planet_rotation(),
            starfield_model: starfield.build(),
        }
    }
}

impl BackgroundView for OdysseyBackgroundView {
    fn draw(&self, resources: &Resources) {
        let _depth = Depth::new();
        let v_matrix = Matrix4::look_at(
            Point3::new(0.0, 0.0, 100.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );
        let light_dir_world_space = Vector3::new(-3.0, 3.0, 10.0);

        let m_matrix = Matrix4::from_translation(vec3(-38.0, -34.0, 0.0))
            * Matrix4::from_angle_x(PLANET_TILT_X)
            * Matrix4::from_angle_z(PLANET_TILT_Z)
            * Matrix4::from_angle_y(Deg(-0.1) * (self.planet_rotation as f32));
        resources.shaders().scene().render(
            &self.p_matrix,
            &v_matrix,
            light_dir_world_space,
            &m_matrix,
            resources.textures().red_planet(),
            &self.planet_model,
        );

        resources.shaders().scene().render(
            &self.p_matrix,
            &v_matrix,
            light_dir_world_space,
            &Matrix4::identity(),
            resources.textures().starfield(),
            &self.starfield_model,
        );
    }

    fn on_event(&mut self, event: &Event, ui: &mut Ui) {
        match event {
            Event::ClockTick(_) => {
                let new_planet_rotation = get_planet_rotation();
                if self.planet_rotation != new_planet_rotation {
                    self.planet_rotation = new_planet_rotation;
                    ui.request_redraw();
                }
            }
            _ => {}
        }
    }
}

//===========================================================================//

fn get_planet_rotation() -> i32 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    let seconds = (duration.as_secs() % 3600) as i32;
    let subseconds = (duration.subsec_millis() / 200) as i32;
    5 * seconds + subseconds
}

//===========================================================================//
