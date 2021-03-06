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

use super::chip::ChipType;
use super::wire::WireShape;
use crate::geom::{CoordsDelta, CoordsSize, Direction, Orientation};
use serde::de::Error;
use std::cmp::Ordering;
use std::collections::{btree_map, BTreeMap};
use std::fs;
use std::i32;
use std::path::Path;
use toml;

//===========================================================================//

#[derive(Clone, Deserialize, Serialize)]
pub struct CircuitData {
    pub size: CoordsSize,
    pub chips: CircuitChipData,
    pub wires: CircuitWireData,
}

impl CircuitData {
    pub fn new(width: i32, height: i32) -> CircuitData {
        CircuitData {
            size: CoordsSize::new(width, height),
            chips: CircuitChipData(BTreeMap::new()),
            wires: CircuitWireData(BTreeMap::new()),
        }
    }

    pub fn load(path: &Path) -> Result<CircuitData, String> {
        let bytes = fs::read(path).map_err(|err| {
            format!("Could not read circuit file from {:?}: {}", path, err)
        })?;
        toml::from_slice(&bytes)
            .map_err(|err| format!("Could not deserialize circuit: {}", err))
    }

    fn serialize_toml(&self) -> Result<Vec<u8>, String> {
        toml::to_vec(self)
            .map_err(|err| format!("Could not serialize circuit: {}", err))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let bytes = self.serialize_toml()?;
        fs::write(path, bytes).map_err(|err| {
            format!("Could not write circuit file to {:?}: {}", path, err)
        })?;
        Ok(())
    }

    pub fn deserialize_from_string(
        string: &str,
    ) -> Result<CircuitData, String> {
        toml::from_str(string)
            .map_err(|err| format!("Could not deserialize circuit: {}", err))
    }

    pub fn serialize_to_string(&self) -> Result<String, String> {
        toml::to_string(self)
            .map_err(|err| format!("Could not serialize circuit: {}", err))
    }
}

//===========================================================================//

#[derive(Clone)]
pub struct CircuitChipData(BTreeMap<(i32, i32), (ChipType, Orientation)>);

impl CircuitChipData {
    pub fn insert(
        &mut self,
        delta: CoordsDelta,
        ctype: ChipType,
        orient: Orientation,
    ) {
        self.0.insert((delta.x, delta.y), (ctype, orient));
    }

    pub fn iter(&self) -> CircuitChipDataIter {
        CircuitChipDataIter { inner: self.0.iter() }
    }
}

impl<'d> serde::Deserialize<'d> for CircuitChipData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        let map = BTreeMap::<&str, &str>::deserialize(deserializer)?;
        let mut chips = BTreeMap::new();
        for (key, chip_str) in map.into_iter() {
            let coords = key_string_delta(key).ok_or_else(|| {
                D::Error::custom(format!("Invalid coords key: {:?}", key))
            })?;
            let mut items = chip_str.splitn(2, '-');
            let orient_str = items.next().ok_or_else(|| {
                D::Error::custom(format!("Invalid chip spec: {:?}", chip_str))
            })?;
            let orient = orient_str.parse::<Orientation>().map_err(|_| {
                D::Error::custom(format!("Invalid chip spec: {:?}", chip_str))
            })?;
            let ctype_str = items.next().ok_or_else(|| {
                D::Error::custom(format!("Invalid chip spec: {:?}", chip_str))
            })?;
            let ctype = ctype_str.parse::<ChipType>().map_err(|_| {
                D::Error::custom(format!("Invalid chip spec: {:?}", chip_str))
            })?;
            if items.next().is_some() {
                return Err(D::Error::custom(format!(
                    "Invalid chip spec: {:?}",
                    chip_str
                )));
            }
            chips.insert((coords.x, coords.y), (ctype, orient));
        }
        Ok(CircuitChipData(chips))
    }
}

impl serde::Serialize for CircuitChipData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0
            .iter()
            .map(|(&(x, y), &(ctype, orient))| {
                (
                    delta_key_string(CoordsDelta::new(x, y)),
                    format!("{}-{}", orient, ctype),
                )
            })
            .collect::<BTreeMap<String, String>>()
            .serialize(serializer)
    }
}

pub struct CircuitChipDataIter<'a> {
    inner: btree_map::Iter<'a, (i32, i32), (ChipType, Orientation)>,
}

impl<'a> Iterator for CircuitChipDataIter<'a> {
    type Item = (CoordsDelta, ChipType, Orientation);

    fn next(&mut self) -> Option<(CoordsDelta, ChipType, Orientation)> {
        self.inner.next().map(|(&(x, y), &(ctype, orient))| {
            (CoordsDelta::new(x, y), ctype, orient)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

//===========================================================================//

#[derive(Clone)]
pub struct CircuitWireData(BTreeMap<(i32, i32, OrderedDir), WireShape>);

impl CircuitWireData {
    pub fn insert(
        &mut self,
        delta: CoordsDelta,
        dir: Direction,
        shape: WireShape,
    ) {
        self.0.insert((delta.x, delta.y, OrderedDir(dir)), shape);
    }

    pub fn iter(&self) -> CircuitWireDataIter {
        CircuitWireDataIter { inner: self.0.iter() }
    }
}

impl<'d> serde::Deserialize<'d> for CircuitWireData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        let map = BTreeMap::<&str, WireShape>::deserialize(deserializer)?;
        let mut wires = BTreeMap::new();
        for (key, shape) in map.into_iter() {
            let (delta, dir) = key_string_location(key).ok_or_else(|| {
                D::Error::custom(format!("Invalid location key: {:?}", key))
            })?;
            wires.insert((delta.x, delta.y, OrderedDir(dir)), shape);
        }
        Ok(CircuitWireData(wires))
    }
}

impl serde::Serialize for CircuitWireData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0
            .iter()
            .map(|(&(x, y, OrderedDir(dir)), &shape)| {
                (location_key_string(CoordsDelta::new(x, y), dir), shape)
            })
            .collect::<BTreeMap<String, WireShape>>()
            .serialize(serializer)
    }
}

pub struct CircuitWireDataIter<'a> {
    inner: btree_map::Iter<'a, (i32, i32, OrderedDir), WireShape>,
}

impl<'a> Iterator for CircuitWireDataIter<'a> {
    type Item = (CoordsDelta, Direction, WireShape);

    fn next(&mut self) -> Option<(CoordsDelta, Direction, WireShape)> {
        self.inner.next().map(|(&(x, y, OrderedDir(dir)), &shape)| {
            (CoordsDelta::new(x, y), dir, shape)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OrderedDir(Direction);

impl PartialOrd for OrderedDir {
    fn partial_cmp(&self, other: &OrderedDir) -> Option<Ordering> {
        (self.0 as u32).partial_cmp(&(other.0 as u32))
    }
}

impl Ord for OrderedDir {
    fn cmp(&self, other: &OrderedDir) -> Ordering {
        (self.0 as u32).cmp(&(other.0 as u32))
    }
}

//===========================================================================//

pub(super) fn delta_key_string(delta: CoordsDelta) -> String {
    let xsign = if delta.x < 0 { 'm' } else { 'p' };
    let ysign = if delta.y < 0 { 'm' } else { 'p' };
    format!("{}{}{}{}", xsign, delta.x.abs(), ysign, delta.y.abs())
}

pub(super) fn key_string_delta(key: &str) -> Option<CoordsDelta> {
    let mut chars = key.chars();
    let xsign_chr = chars.next();
    let xsign: i32 = match xsign_chr {
        Some('m') => -1,
        Some('p') => 1,
        _ => return None,
    };
    let mut ysign_chr = None;
    let mut x: u64 = 0;
    while let Some(chr) = chars.next() {
        if let Some(digit) = chr.to_digit(10) {
            x = 10 * x + (digit as u64);
            if x > (i32::MAX as u64) {
                return None;
            }
        } else {
            ysign_chr = Some(chr);
            break;
        }
    }
    let ysign: i32 = match ysign_chr {
        Some('m') => -1,
        Some('p') => 1,
        _ => return None,
    };
    let mut y: u64 = 0;
    while let Some(chr) = chars.next() {
        if let Some(digit) = chr.to_digit(10) {
            y = 10 * y + (digit as u64);
            if y > (i32::MAX as u64) {
                return None;
            }
        } else {
            return None;
        }
    }
    return Some(CoordsDelta::new(xsign * (x as i32), ysign * (y as i32)));
}

fn location_key_string(delta: CoordsDelta, dir: Direction) -> String {
    let dir_chr = match dir {
        Direction::East => 'e',
        Direction::South => 's',
        Direction::West => 'w',
        Direction::North => 'n',
    };
    format!("{}{}", delta_key_string(delta), dir_chr)
}

fn key_string_location(key: &str) -> Option<(CoordsDelta, Direction)> {
    let mut string = key.to_string();
    let dir = match string.pop() {
        Some('e') => Direction::East,
        Some('s') => Direction::South,
        Some('w') => Direction::West,
        Some('n') => Direction::North,
        _ => return None,
    };
    if let Some(delta) = key_string_delta(&string) {
        Some((delta, dir))
    } else {
        None
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{ChipType, CircuitData, OrderedDir, WireShape};
    use crate::geom::{CoordsDelta, CoordsSize, Direction, Orientation};
    use toml;

    #[test]
    fn serialize_circuit_data() {
        let mut data = CircuitData::new(8, 5);
        data.chips.insert(
            CoordsDelta::new(2, 3),
            ChipType::Break(true),
            Orientation::default(),
        );
        data.chips.insert(
            CoordsDelta::new(1, 3),
            ChipType::Button(None),
            Orientation::default().flip_vert(),
        );
        data.chips.insert(
            CoordsDelta::new(1, 4),
            ChipType::Comment(*b"Press"),
            Orientation::default().rotate_ccw(),
        );
        data.wires.insert(
            CoordsDelta::new(1, 3),
            Direction::East,
            WireShape::Stub,
        );
        data.wires.insert(
            CoordsDelta::new(2, 3),
            Direction::West,
            WireShape::Stub,
        );
        let bytes = data.serialize_toml().unwrap();
        assert_eq!(
            String::from_utf8(bytes).unwrap().as_str(),
            "size = [8, 5]\n\n\
             [chips]\n\
             p1p3 = \"t0-Button\"\n\
             p1p4 = \"f3-Comment('Press')\"\n\
             p2p3 = \"f0-Break(true)\"\n\n\
             [wires]\n\
             p1p3e = \"Stub\"\n\
             p2p3w = \"Stub\"\n"
        );
    }

    #[test]
    fn deserialize_circuit_data() {
        let toml = "size = [8, 5]\n\n\
                    [chips]\n\
                    p1p3 = \"t0-Button\"\n\
                    p1p4 = \"f3-Comment('Press')\"\n\
                    p2p3 = \"f0-Break(true)\"\n\n\
                    [wires]\n\
                    p1p3e = \"Stub\"\n\
                    p2p3w = \"Stub\"\n";
        let data: CircuitData = toml::from_slice(toml.as_bytes()).unwrap();
        assert_eq!(data.size, CoordsSize::new(8, 5));
        assert_eq!(
            data.chips.0,
            vec![
                (
                    (1, 3),
                    (
                        ChipType::Button(None),
                        Orientation::default().flip_vert()
                    )
                ),
                (
                    (1, 4),
                    (
                        ChipType::Comment(*b"Press"),
                        Orientation::default().rotate_ccw()
                    )
                ),
                ((2, 3), (ChipType::Break(true), Orientation::default())),
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(
            data.wires.0,
            vec![
                ((1, 3, OrderedDir(Direction::East)), WireShape::Stub),
                ((2, 3, OrderedDir(Direction::West)), WireShape::Stub),
            ]
            .into_iter()
            .collect()
        );
    }
}

//===========================================================================//
