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

use super::index::IndexBuffer;
use super::vertex::{Primitive, VertexArray, VertexBuffer};

//===========================================================================//

pub struct HeightmapModel {
    varray: VertexArray,
    _vbuffer: VertexBuffer<f32>,
    ibuffer: IndexBuffer<u16>,
}

impl HeightmapModel {
    pub fn new(dim: u16) -> HeightmapModel {
        debug_assert!(dim >= 2);
        debug_assert!(dim <= 255);
        let mut vertices =
            Vec::<f32>::with_capacity((2 * (dim + 1) * (dim + 1)) as usize);
        let step = 1.0 / (dim as f32);
        for row in 0..(dim + 1) {
            let y = step * (row as f32);
            for col in 0..(dim + 1) {
                let x = step * (col as f32);
                vertices.push(x);
                vertices.push(y);
            }
        }
        let vbuffer = VertexBuffer::new(&vertices);

        let mut indices = Vec::<u16>::with_capacity(6 * (dim as usize) *
                                                        (dim as usize));
        for row in 0..dim {
            for col in 0..dim {
                let i1 = (dim + 1) * row + col;
                let i2 = i1 + 1;
                let i3 = i1 + dim + 1;
                let i4 = i3 + 1;
                indices.extend(&[i2, i1, i3, i2, i3, i4]);
            }
        }
        let ibuffer = IndexBuffer::new(&indices);

        let varray = VertexArray::new(1);
        varray.bind();
        vbuffer.attribf(0, 2, 0, 0);

        HeightmapModel {
            varray,
            _vbuffer: vbuffer,
            ibuffer,
        }
    }

    pub fn draw(&self) {
        self.varray.bind();
        self.varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }
}

//===========================================================================//
