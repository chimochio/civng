use hexpos::AxialPos;

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

    pub fn ispassable(&self, pos: AxialPos) -> bool {
        if pos.q < 0 || pos.r < 0 || pos.q >= self.width || pos.r >= self.height {
            // out of bounds
            return false
        }
        !self.data[(pos.r * self.width + pos.q) as usize]
    }

}

