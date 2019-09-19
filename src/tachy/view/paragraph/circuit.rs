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

use super::super::chip::ChipModel;
use super::super::wire::WireModel;
use super::types::{CompiledPiece, ParserPiece, ParserPieceSplit};
use crate::tachy::geom::{
    Color4, Coords, CoordsSize, Direction, MatrixExt, Orientation,
};
use crate::tachy::gui::Resources;
use crate::tachy::save::{ChipType, CircuitData, WireShape};
use crate::tachy::state::{
    self, ChipExt, PortColor, PortConstraint, PortFlow, WireInfo, WireSize,
};
use cgmath::{Matrix4, Vector2};
use std::collections::HashMap;
use std::usize;

//===========================================================================//

const CIRCUIT_GRID_CELL_SIZE: f32 = 64.0;
const CIRCUIT_MARGIN_HORZ: f32 = 8.0;

//===========================================================================//

pub struct ParserCircuitPiece {
    num_millis: usize,
    data: CircuitData,
}

impl ParserCircuitPiece {
    pub fn new(num_millis: usize, data: CircuitData) -> ParserCircuitPiece {
        ParserCircuitPiece { num_millis, data }
    }
}

impl ParserPiece for ParserCircuitPiece {
    fn is_empty(&self) -> bool {
        false
    }

    fn width(&self, _font_size: f32) -> f32 {
        (self.data.bounds.width as f32) * CIRCUIT_GRID_CELL_SIZE
            + 2.0 * CIRCUIT_MARGIN_HORZ
    }

    fn height(&self, _font_size: f32) -> f32 {
        (self.data.bounds.height as f32) * CIRCUIT_GRID_CELL_SIZE
    }

    fn num_millis(&self) -> usize {
        self.num_millis
    }

    fn split(
        &mut self,
        font_size: f32,
        remaining_width: f32,
    ) -> ParserPieceSplit {
        if remaining_width >= self.width(font_size) {
            ParserPieceSplit::AllFits
        } else {
            ParserPieceSplit::NoneFits(None)
        }
    }

    fn compile(
        &mut self,
        x_offset: f32,
        y_offset: f32,
    ) -> Box<dyn CompiledPiece> {
        let delta = Coords::new(0, 0) - self.data.bounds.top_left();
        let chips: HashMap<Coords, (ChipType, Orientation)> = self
            .data
            .chips
            .iter()
            .map(|(coords, ctype, orient)| (coords + delta, (ctype, orient)))
            .collect();
        let mut fragments: HashMap<(Coords, Direction), (WireShape, usize)> =
            self.data
                .wires
                .iter()
                .map(|(coords, dir, shape)| {
                    ((coords, dir), (shape, usize::MAX))
                })
                .collect();
        // TODO: Perform fragment repair, like in EditGrid::from_circuit_data,
        // so that we don't need to put every fragment in the paragraph spec.
        let ports: HashMap<(Coords, Direction), (PortFlow, PortColor)> = chips
            .iter()
            .flat_map(|(&coords, &(ctype, orient))| {
                ctype
                    .ports(coords, orient)
                    .into_iter()
                    .map(|port| (port.loc(), (port.flow, port.color)))
            })
            .collect();
        let mut wires = state::group_wires(&ports, &mut fragments);
        let _errors = state::recolor_wires(&mut wires);
        let wires_for_ports = state::map_ports_to_wires(&wires);
        let constraints: Vec<PortConstraint> = chips
            .iter()
            .flat_map(|(&coords, &(ctype, orient))| {
                ctype.constraints(coords, orient)
            })
            .collect();
        let _errors = state::determine_wire_sizes(
            &mut wires,
            &wires_for_ports,
            constraints,
        );
        let piece = CompiledCircuitPiece {
            offset: Vector2::new(x_offset + CIRCUIT_MARGIN_HORZ, y_offset),
            num_millis: self.num_millis,
            size: self.data.bounds.size(),
            chips,
            fragments,
            wires,
        };
        Box::new(piece)
    }
}

//===========================================================================//

struct CompiledCircuitPiece {
    offset: Vector2<f32>,
    num_millis: usize,
    size: CoordsSize,
    chips: HashMap<Coords, (ChipType, Orientation)>,
    fragments: HashMap<(Coords, Direction), (WireShape, usize)>,
    wires: Vec<WireInfo>,
}

impl CompiledPiece for CompiledCircuitPiece {
    fn height(&self, _font_size: f32) -> f32 {
        (self.size.height as f32) * CIRCUIT_GRID_CELL_SIZE
    }

    fn add_x_offset(&mut self, x_offset: f32) {
        self.offset.x += x_offset;
    }

    fn add_y_offset(&mut self, y_offset: f32) {
        self.offset.y += y_offset;
    }

    fn draw(
        &self,
        resources: &Resources,
        paragraph_matrix: &Matrix4<f32>,
        _font_size: f32,
        millis_remaining: &mut usize,
    ) -> bool {
        if *millis_remaining < self.num_millis {
            return false;
        }
        *millis_remaining -= self.num_millis;
        let grid_matrix = paragraph_matrix
            * Matrix4::trans2v(self.offset)
            * Matrix4::from_scale(CIRCUIT_GRID_CELL_SIZE);
        // Draw wires:
        for (&(coords, dir), &(shape, index)) in self.fragments.iter() {
            let info = &self.wires[index];
            let size = info.size.lower_bound().unwrap_or(WireSize::One);
            WireModel::draw_fragment(
                resources,
                &grid_matrix,
                coords,
                dir,
                shape,
                info.color,
                size,
                &Color4::TRANSPARENT,
            );
        }
        // Draw chips:
        for (&coords, &(ctype, orient)) in self.chips.iter() {
            ChipModel::draw_chip(
                resources,
                &grid_matrix,
                coords,
                ctype,
                orient,
                None,
            );
        }
        return true;
    }

    fn debug_string(&self) -> String {
        "[Circuit]".to_string()
    }
}

//===========================================================================//
