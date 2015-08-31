/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

//! Manages map information (terrain and stuff).
//!
//! Map coordinates are in `OffsetPos` because that's the easiest layout for serialization.

use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::io::Read;
use std::slice::Iter;

use num::integer::Integer;

use hexpos::{Pos, OffsetPos, PosPath};

/// Terrain type
///
/// Each tile in civng has a terrain type, which is represented by this structure.
#[derive(Copy, Clone)]
pub enum Terrain {
    Plain,
    Grassland,
    Desert,
    Hill,
    Mountain,
    Water,
    OutOfBounds,
}

impl Terrain {
    pub fn all() -> [Terrain; 6] {
        [
            Terrain::Plain,
            Terrain::Grassland,
            Terrain::Desert,
            Terrain::Hill,
            Terrain::Mountain,
            Terrain::Water,
        ]
    }

    /// Returns the character representing a particular terrain on screen.
    pub fn map_char(&self) -> char {
        match *self {
            Terrain::Plain => '\'',
            Terrain::Grassland => '"',
            Terrain::Desert => ' ',
            Terrain::Hill => '^',
            Terrain::Mountain => 'A',
            Terrain::Water => '~',
            Terrain::OutOfBounds => '?',
        }
    }

    pub fn name(&self) -> &str {
        match *self {
            Terrain::Plain => "Plain",
            Terrain::Grassland => "Grassland",
            Terrain::Desert => "Desert",
            Terrain::Hill => "Hill",
            Terrain::Mountain => "Mountain",
            Terrain::Water => "Water",
            Terrain::OutOfBounds => "Out of bounds",
        }
    }

    pub fn defense_modifier(&self) -> i8 {
        match *self {
            Terrain::Plain => 0,
            Terrain::Grassland => 0,
            Terrain::Desert => 0,
            Terrain::Hill => 25,
            Terrain::Mountain => 0,
            Terrain::Water => 0,
            Terrain::OutOfBounds => 0,
        }
    }

    /// Returns whether the terrain is passable by our moving unit.
    pub fn is_passable(&self) -> bool {
        match *self {
            Terrain::Mountain | Terrain::Water | Terrain::OutOfBounds => false,
            _ => true,
        }
    }

    /// Returns how much movement points it costs to move on that terrain.
    pub fn movement_cost(&self) -> u8 {
        match *self {
            Terrain::Hill => 2,
            _ => 1,
        }
    }
}

// You would think that it would be simpler for fn tiles() to simply return an enumerated and
// mapped iterator rather than having this whole struct, right? Think again! There's all kinds
// of complications when you try to do that (I spent *hours* on this), the fatal one being
// closure lifetime. Hence, this iterator struct.

pub struct TilesIterator<'a> {
    map_width: i32,
    counter: usize,
    terrain_iter: Iter<'a, Terrain>,
}

impl<'a> TilesIterator<'a> {
    pub fn new(terrain_iter: Iter<Terrain>, map_width: i32) -> TilesIterator{
        TilesIterator {
            map_width: map_width,
            counter: 0,
            terrain_iter: terrain_iter,
        }
    }
}

impl<'a> Iterator for TilesIterator<'a> {
    type Item = (Pos, Terrain);

    fn next(&mut self) -> Option<(Pos, Terrain)> {
        match self.terrain_iter.next() {
            Some(t) => {
                let index = self.counter as i32;
                let (y, x) = index.div_rem(&self.map_width);
                self.counter += 1;
                Some((OffsetPos::new(x, y).to_pos(), *t))
            }
            None => None,
        }
    }
}

/// Map of terrain tiles
///
/// top left corner is (0, 0) in offset pos.
pub struct TerrainMap {
    width: i32,
    height: i32,
    data: Vec<Terrain>, // sequence of rows, then cols. len == width * height.
}

impl TerrainMap {
    pub fn new(width: i32, height: i32, data: Vec<Terrain>) -> TerrainMap {
        if data.len() != (width * height) as usize {
            panic!("Inconsistent TerrainMap data");
        }
        TerrainMap {
            width: width,
            height: height,
            data: data,
        }
    }

    /// Creates a map filled with grassland.
    ///
    /// Useful for testing.
    pub fn empty_map(width: i32, height: i32) -> TerrainMap {
        TerrainMap::new(
            width,
            height,
            vec![Terrain::Grassland; (width * height) as usize],
        )
    }

    /// Loads terrain map from text file.
    ///
    /// The file is a series of lines of the same length, each character representing a terrain
    /// tile. That character is defined by `Terrain.map_char()`.
    ///
    /// If the character can't be recognized, it defaults as Water.
    ///
    /// Panics if anything goes wrong.
    pub fn fromfile(path: &Path) -> TerrainMap {
        let fp = File::open(path).unwrap();
        let mut width: Option<i32> = None;
        let mut chcount: i32 = 0;
        let allterrain = Terrain::all();
        let char2terrain = HashMap::<char, &Terrain>::from_iter(
            allterrain.iter().map(|t| (t.map_char(), t))
        );
        let mut data: Vec<Terrain> = Vec::new();
        for byte in fp.bytes() {
            let ch = match byte {
                Ok(ch) => ch as char,
                Err(_) => break,
            };
            if ch == '\n' {
                match width {
                    Some(w) => { assert!(chcount == w) },
                    None => { width = Some(data.len() as i32) },
                }
                chcount = 0;
            }
            else {
                chcount += 1;
                match char2terrain.get(&ch) {
                    Some(t) => { data.push(**t) },
                    None => { data.push(Terrain::Water) },
                };
            }
        }
        let height = (data.len() as i32) / width.unwrap();
        TerrainMap::new(width.unwrap(), height, data)
    }

    pub fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    /// Returns terrain at a particular pos.
    ///
    /// We take care of converting `Pos` into `OffsetPos`. If out of bounds, returns Water.
    pub fn get_terrain(&self, pos: Pos) -> Terrain {
        let opos = pos.to_offset_pos();
        if opos.x < 0 || opos.y < 0 || opos.x >= self.width || opos.y >= self.height {
            // out of bounds
            return Terrain::OutOfBounds
        }
        self.data[(opos.y * self.width + opos.x) as usize]
    }

    pub fn tiles(&self) -> TilesIterator {
        TilesIterator::new(self.data.iter(), self.width)
    }

    pub fn movement_cost(&self, path: &PosPath) -> u8 {
        path.stack()[1..].iter().fold(0, |acc, &p| acc + self.get_terrain(p).movement_cost())
    }
}

