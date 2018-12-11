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

use super::wire::WireShape;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use toml;

//===========================================================================//

#[derive(Deserialize, Serialize)]
pub struct CircuitData {
    pub bounds: (i32, i32, i32, i32),
    pub chips: BTreeMap<String, String>,
    pub wires: BTreeMap<String, WireShape>,
}

impl CircuitData {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> CircuitData {
        CircuitData {
            bounds: (x, y, width, height),
            chips: BTreeMap::new(),
            wires: BTreeMap::new(),
        }
    }

    fn serialize(&self) -> Result<Vec<u8>, String> {
        toml::to_vec(self)
            .map_err(|err| format!("Could not serialize circuit: {}", err))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let bytes = self.serialize()?;
        fs::write(path, bytes)
            .map_err(|err| {
                         format!("Could not write circuit file to {:?}: {}",
                                 path,
                                 err)
                     })?;
        Ok(())
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{CircuitData, WireShape};

    #[test]
    fn serialize_circuit_data() {
        let mut data = CircuitData::new(-2, -1, 8, 5);
        data.chips.insert("m1p2".to_string(), "f0-Button".to_string());
        data.chips.insert("p0p2".to_string(), "t0-Break".to_string());
        data.wires.insert("m1p2e".to_string(), WireShape::Stub);
        data.wires.insert("p0p2w".to_string(), WireShape::Stub);
        let bytes = data.serialize().unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap().as_str(),
                   "bounds = [-2, -1, 8, 5]\n\n\
                    [chips]\n\
                    m1p2 = \"f0-Button\"\n\
                    p0p2 = \"t0-Break\"\n\n\
                    [wires]\n\
                    m1p2e = \"Stub\"\n\
                    p0p2w = \"Stub\"\n");
    }
}

//===========================================================================//
