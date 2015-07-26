use hexpos::AxialPos;

pub enum Terrain {
    Plain,
    Mountain,
    Water,
}

impl Terrain {
    pub fn map_char(&self) -> char {
        match *self {
            Terrain::Plain => ' ',
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
    data: Vec<bool>, // sequence of rows, then cols. len == width * height.
}

impl TerrainMap {
    pub fn new(width: i32, height: i32, data: Vec<bool>) -> TerrainMap {
        if data.len() != (width * height) as usize {
            panic!("Inconsistent TerrainMap data");
        }
        TerrainMap {
            width: width,
            height: height,
            data: data,
        }
    }

    pub fn get_terrain(&self, pos: AxialPos) -> Terrain {
        if pos.q < 0 || pos.r < 0 || pos.q >= self.width || pos.r >= self.height {
            // out of bounds
            return Terrain::Water
        }
        if self.data[(pos.r * self.width + pos.q) as usize] {
            Terrain::Mountain
        }
        else {
            Terrain::Plain
        }
    }
}

