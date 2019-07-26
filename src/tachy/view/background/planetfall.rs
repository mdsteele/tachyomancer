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
use cgmath::{self, Deg, Matrix4, Point3, Vector3, vec3};
use tachy::geom::{Color3, Rect, RectSize};
use tachy::gl::{Depth, HeightmapModel, Model, ModelBuilder,
                ModelBuilderContext};
use tachy::gui::{Event, Keycode, Resources, Ui};

//===========================================================================//

pub struct PlanetfallBackgroundView {
    p_matrix: Matrix4<f32>,
    habitat_model: Model,
    terrain_model: HeightmapModel,
    sky_model: Model,
    look_y: f32,
}

impl PlanetfallBackgroundView {
    pub fn new(screen_size: RectSize<f32>) -> PlanetfallBackgroundView {
        let aspect = screen_size.width / screen_size.height;
        let p_matrix = cgmath::perspective(Deg(45.0), aspect, 0.1, 1000.0);

        let mut habitat = ModelBuilder::new();
        make_habitat(&mut habitat.context());

        let mut sky = ModelBuilder::new();
        {
            let mut ctx = sky.context();
            ctx.plane(Point3::new(0.0, 0.0, -100.0),
                      RectSize::new(300.0, 300.0),
                      Vector3::unit_z(),
                      Color3::WHITE);
        }

        PlanetfallBackgroundView {
            p_matrix,
            habitat_model: habitat.build(),
            terrain_model: HeightmapModel::new(128),
            sky_model: sky.build(),
            look_y: 8.8,
        }
    }
}

impl BackgroundView for PlanetfallBackgroundView {
    fn draw(&self, resources: &Resources) {
        let _depth = Depth::new();
        let v_matrix = Matrix4::look_at(Point3::new(0.0, 15.0, 90.0),
                                        Point3::new(0.0, self.look_y, 0.0),
                                        Vector3::unit_y());
        let light_dir_world_space = Vector3::new(-3.0, 30.0, 10.0);

        let shader = resources.shaders().scene();
        let m_matrix = Matrix4::from_translation(vec3(15.0, 0.0, 35.0));
        shader.render(&self.p_matrix,
                      &v_matrix,
                      light_dir_world_space,
                      &m_matrix,
                      resources.textures().white(),
                      &self.habitat_model);

        let m_matrix = Matrix4::from_translation(vec3(0.0, 0.0, -100.0)) *
            Matrix4::from_angle_y(Deg(-145.0)) *
            Matrix4::from_nonuniform_scale(300.0, 30.0, 300.0) *
            Matrix4::from_translation(vec3(-0.5, 0.0, -0.5));
        let shader = resources.shaders().heightmap();
        shader.render(&self.p_matrix,
                      &v_matrix,
                      light_dir_world_space,
                      &m_matrix,
                      resources.textures().valley_heightmap(),
                      Rect::new(0.0, -0.5, 2.0, 2.0),
                      resources.textures().red_desert(),
                      Rect::new(0.0, 0.0, 2.0, 2.0),
                      &self.terrain_model);

        let shader = resources.shaders().scene();
        shader.render(&self.p_matrix,
                      &v_matrix,
                      light_dir_world_space,
                      &Matrix4::from_scale(5.0),
                      resources.textures().starfield(),
                      &self.sky_model);
    }

    fn on_event(&mut self, event: &Event, ui: &mut Ui) {
        match event {
            Event::KeyDown(key) => {
                match key.code {
                    Keycode::Up => {
                        self.look_y += 0.1;
                    }
                    Keycode::Down => {
                        self.look_y -= 0.1;
                    }
                    _ => {}
                }
                debug_log!("look_y = {}", self.look_y);
                ui.request_redraw();
            }
            _ => {}
        }
    }
}

//===========================================================================//

const ORIGIN: Point3<f32> = Point3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

fn make_habitat(ctx: &mut ModelBuilderContext) {
    make_dome(ctx, 3.0);
    {
        let mut ctx = make_tunnel(ctx, 7.0, Deg(-45.0));
        make_dome(&mut ctx, 2.0);
        {
            let mut ctx = make_tunnel(&mut ctx, 8.0, Deg(-45.0));
            make_dome(&mut ctx, 1.5);
        }
        {
            let mut ctx = make_tunnel(&mut ctx, 6.0, Deg(45.0));
            make_dome(&mut ctx, 1.5);
        }
    }
    {
        let mut ctx = make_tunnel(ctx, 10.0, Deg(-115.0));
        make_dome(&mut ctx, 2.0);
    }
    {
        let mut ctx = make_tunnel(ctx, 8.0, Deg(110.0));
        make_dome(&mut ctx, 2.0);
        {
            let mut ctx = make_tunnel(&mut ctx, 6.0, Deg(45.0));
            make_dome(&mut ctx, 1.5);
        }
    }
}

fn make_dome(ctx: &mut ModelBuilderContext, radius: f32) {
    let cyl_height = 1.0;
    ctx.cylinder(Point3::new(0.0, -3.0, 0.0),
                 Point3::new(0.0, cyl_height, 0.0),
                 radius,
                 20,
                 Color3::WHITE);
    ctx.with_transform(Matrix4::from_translation(vec3(0.0, cyl_height, 0.0)) *
                           Matrix4::from_nonuniform_scale(1.0, 0.5, 1.0))
        .sphere(ORIGIN, radius, 20, Color3::WHITE);
}

fn make_tunnel<'a>(ctx: &'a mut ModelBuilderContext, length: f32,
                   angle: Deg<f32>)
                   -> ModelBuilderContext<'a> {
    let mut subctx = ctx.with_transform(Matrix4::from_angle_y(angle));
    subctx
        .with_transform(Matrix4::from_nonuniform_scale(1.0, 1.5, 1.0))
        .cylinder(ORIGIN,
                  Point3::new(length, 0.0, 0.0),
                  1.0,
                  20,
                  Color3::new(0.7, 0.7, 0.7));
    subctx.transformed(Matrix4::from_translation(vec3(length, 0.0, 0.0)))
}

//===========================================================================//
