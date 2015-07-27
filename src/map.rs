use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::io::Read;

use hexpos::Pos;

#[derive(Copy, Clone)]
pub enum Terrain {
    Plain,
    Mountain,
    Water,
}

impl Terrain {
    pub fn all() -> [Terrain; 3] {
        [
            Terrain::Plain,
            Terrain::Mountain,
            Terrain::Water,
        ]
    }

    pub fn map_char(&self) -> char {
        match *self {
            Terrain::Plain => '.',
            Terrain::Mountain => '^',
            Terrain::Water => '~',
        }
    }

    pub fn is_passable(&self) -> bool {
        match *self {
            Terrain::Plain => true,
            Terrain::Mountain | Terrain::Water => false
        }
    }
}

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

    pub fn get_terrain(&self, pos: Pos) -> Terrain {
        let opos = pos.to_offset_pos();
        if opos.x < 0 || opos.y < 0 || opos.x >= self.width || opos.y >= self.height {
            // out of bounds
            return Terrain::Water
        }
        self.data[(opos.y * self.width + opos.x) as usize]
    }
}

