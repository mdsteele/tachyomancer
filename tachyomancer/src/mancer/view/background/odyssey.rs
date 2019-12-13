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
use crate::mancer::gl::{Depth, Model, ModelBuilder, ModelBuilderContext};
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
    ship_model: Model,
    starfield_model: Model,
}

impl OdysseyBackgroundView {
    pub fn new(screen_size: RectSize<f32>) -> OdysseyBackgroundView {
        let aspect = screen_size.width / screen_size.height;
        let p_matrix = cgmath::perspective(Deg(45.0), aspect, 0.1, 1000.0);

        let mut planet = ModelBuilder::new();
        planet
            .with_transform(Matrix4::from_scale(PLANET_RADIUS))
            .unit_sphere(24, Color3::WHITE);

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
            ship_model: make_ship_model(),
            starfield_model: starfield.build(),
        }
    }
}

impl BackgroundView for OdysseyBackgroundView {
    fn draw(&self, resources: &Resources) {
        let depth = Depth::enable_with_face_culling(true);

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

        let m_matrix = Matrix4::from_translation(vec3(40.0, -8.0, 0.0))
            * Matrix4::from_angle_y(Deg(20.0))
            * Matrix4::from_angle_x(Deg(10.0))
            * Matrix4::from_scale(1.5);
        resources.shaders().scene().render(
            &self.p_matrix,
            &v_matrix,
            light_dir_world_space,
            &m_matrix,
            resources.textures().brushed_metal(),
            &self.ship_model,
        );

        resources.shaders().scene().render(
            &self.p_matrix,
            &v_matrix,
            light_dir_world_space,
            &Matrix4::identity(),
            resources.textures().starfield(),
            &self.starfield_model,
        );

        depth.disable();
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

fn make_ship_model() -> Model {
    let mut ship = ModelBuilder::new();

    // Chassis:
    {
        let num_steps = 24;
        ship.with_transform(
            Matrix4::from_translation(vec3(0.0, 0.0, -5.0))
                * Matrix4::from_nonuniform_scale(4.0, 3.0, 5.0)
                * Matrix4::from_angle_x(Deg(-90.0)),
        )
        .unit_hemisphere(num_steps, Color3::WHITE);
        let mut ctx =
            ship.with_transform(Matrix4::from_nonuniform_scale(4.0, 3.0, 1.0));
        ctx.cylinder(
            Point3::new(0.0, 0.0, -5.0),
            Point3::new(0.0, 0.0, 5.0),
            1.0,
            num_steps,
            Color3::WHITE,
        );
        ctx.with_transform(
            Matrix4::from_translation(vec3(0.0, 0.0, 5.0))
                * Matrix4::from_angle_x(Deg(90.0)),
        )
        .unit_circle(num_steps, Color3::WHITE);
    }

    // Fuel tanks:
    make_fuel_tank(&mut ship.with_transform(
        Matrix4::from_translation(vec3(-4.0, -3.0, -2.5))
            * Matrix4::from_angle_x(Deg(90.0)),
    ));
    make_fuel_tank(&mut ship.with_transform(
        Matrix4::from_translation(vec3(-4.0, -3.0, 2.5))
            * Matrix4::from_angle_x(Deg(90.0)),
    ));

    ship.build()
}

fn make_fuel_tank(ctx: &mut ModelBuilderContext) {
    let num_steps = 16;

    // Body:
    ctx.cylinder(
        Point3::new(0.0, -1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        num_steps,
        Color3::WHITE,
    );

    // Caps:
    ctx.with_transform(
        Matrix4::from_translation(vec3(0.0, 1.0, 0.0))
            * Matrix4::from_nonuniform_scale(1.0, 0.5, 1.0),
    )
    .unit_hemisphere(num_steps, Color3::WHITE);
    ctx.with_transform(
        Matrix4::from_translation(vec3(0.0, -1.0, 0.0))
            * Matrix4::from_angle_x(Deg(180.0))
            * Matrix4::from_nonuniform_scale(1.0, 0.5, 1.0),
    )
    .unit_hemisphere(num_steps, Color3::WHITE);

    // Bands:
    let band_offset = 0.7;
    let band_width = 0.2;
    let band_radius = 1.05;
    let band_color = Color3::new(0.6, 0.6, 1.0);
    for index in 0..2 {
        let y = -band_offset + 2.0 * band_offset * (index as f32);
        ctx.cylinder(
            Point3::new(0.0, y - band_width * 0.5, 0.0),
            Point3::new(0.0, y + band_width * 0.5, 0.0),
            band_radius,
            num_steps,
            band_color,
        );
        ctx.with_transform(
            Matrix4::from_translation(vec3(0.0, y + band_width * 0.5, 0.0))
                * Matrix4::from_scale(band_radius),
        )
        .unit_circle(num_steps, band_color);
    }
}

//===========================================================================//
