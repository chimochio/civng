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

use num::integer::Integer;

use hexpos::{Pos, OffsetPos};

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
        }
    }

    /// Returns whether the terrain is passable by our moving unit.
    pub fn is_passable(&self) -> bool {
        match *self {
            Terrain::Mountain | Terrain::Water => false,
            _ => true,
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

    /// Returns terrain at a particular pos.
    ///
    /// We take care of converting `Pos` into `OffsetPos`. If out of bounds, returns Water.
    pub fn get_terrain(&self, pos: Pos) -> Terrain {
        let opos = pos.to_offset_pos();
        if opos.x < 0 || opos.y < 0 || opos.x >= self.width || opos.y >= self.height {
            // out of bounds
            return Terrain::Water
        }
        self.data[(opos.y * self.width + opos.x) as usize]
    }

    pub fn first_passable(&self) -> Pos {
        let (index, _) = self.data.iter().enumerate().find(|&(_, &t)| t.is_passable()).unwrap();
        let (y, x) = (index as i32).div_rem(&self.width);
        OffsetPos::new(x, y).to_pos()
    }
}

