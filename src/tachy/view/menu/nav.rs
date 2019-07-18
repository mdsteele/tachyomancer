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

use cgmath::{self, Deg, Matrix4, Point3, SquareMatrix, Vector3};
use tachy::geom::{Color3, Rect, RectSize};
use tachy::gl::{Depth, Model, ModelBuilder};
use tachy::gui::{Event, Keycode, Resources, Ui};
use tachy::state::GameState;

//===========================================================================//

pub struct NavigationView {
    p_matrix: Matrix4<f32>,
    model: Model,
    m_matrix: Matrix4<f32>,
}

impl NavigationView {
    pub fn new(screen_size: RectSize<f32>, _rect: Rect<i32>, _ui: &mut Ui,
               _state: &GameState)
               -> NavigationView {
        let aspect = screen_size.width / screen_size.height;
        let p_matrix = cgmath::perspective(Deg(45.0), aspect, 0.1, 1000.0);
        let mut model = ModelBuilder::new();
        let p1 = Point3::new(-20.0, -20.0, 0.0);
        let p2 = Point3::new(20.0, -20.0, 0.0);
        let p3 = Point3::new(0.0, 20.0, 0.0);
        let radius = 3.0;
        let faces = 12;
        let color = Color3::new(1.0, 0.6, 0.5);
        model.cylinder(p1, p2, radius, faces, color);
        model.cylinder(p2, p3, radius, faces, color);
        model.cylinder(p3, p1, radius, faces, color);
        model.sphere(p1, radius, faces, color);
        model.sphere(p2, radius, faces, color);
        model.sphere(p3, radius, faces, color);
        NavigationView {
            p_matrix,
            model: model.build(),
            m_matrix: Matrix4::identity(),
        }
    }

    pub fn draw(&self, resources: &Resources, _matrix: &Matrix4<f32>,
                _state: &GameState) {
        let _depth = Depth::new();
        let v_matrix = Matrix4::look_at(Point3::new(0.0, 0.0, 80.0),
                                        Point3::new(0.0, 0.0, 0.0),
                                        Vector3::unit_y());
        let light_dir_world_space = Vector3::new(-3.0, 3.0, 10.0);
        resources.shaders().scene().render(&self.p_matrix,
                                           &v_matrix,
                                           light_dir_world_space,
                                           &self.m_matrix,
                                           &self.model);
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui,
                    _state: &mut GameState) {
        match event {
            Event::KeyDown(key) => {
                match key.code {
                    Keycode::Up => {
                        self.m_matrix = Matrix4::from_angle_x(Deg(-10.0)) *
                            self.m_matrix;
                    }
                    Keycode::Down => {
                        self.m_matrix = Matrix4::from_angle_x(Deg(10.0)) *
                            self.m_matrix;
                    }
                    Keycode::Left => {
                        self.m_matrix = Matrix4::from_angle_y(Deg(-10.0)) *
                            self.m_matrix;
                    }
                    Keycode::Right => {
                        self.m_matrix = Matrix4::from_angle_y(Deg(10.0)) *
                            self.m_matrix;
                    }
                    _ => {}
                }
                ui.request_redraw();
            }
            _ => {}
        }
    }
}

//===========================================================================//
