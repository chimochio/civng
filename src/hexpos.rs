// Awesome resource: http://www.redblobgames.com/grids/hexagons/

#[derive(Copy, Clone, PartialEq)]
pub enum Direction {
    North,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
}

#[derive(Copy, Clone)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32, z: i32) -> Pos {
        Pos {
            x: x,
            y: y,
            z: z,
        }
    }

    pub fn to_axialpos(&self) -> AxialPos {
        AxialPos::new(self.x, self.z)
    }

    pub fn neighbor(&self, direction: Direction) -> Pos {
        let mut p = *self;
        match direction {
            Direction::North => { p.y += 1; p.z -= 1 },
            Direction::NorthEast => { p.x += 1; p.z -= 1 },
            Direction::SouthEast => { p.x += 1; p.y -= 1 },
            Direction::South => { p.z += 1; p.y -= 1 },
            Direction::SouthWest => { p.z += 1; p.x -= 1 },
            Direction::NorthWest => { p.y += 1; p.x -= 1 },
        }
        p
    }

    pub fn fmt(&self) -> String {
        format!("{},{},{}", self.x, self.y, self.z)
    }
}

#[derive(Copy, Clone)]
pub struct AxialPos {
    pub q: i32,
    pub r: i32,
}

impl AxialPos {
    pub fn new(q: i32, r: i32) -> AxialPos {
        AxialPos {
            q: q,
            r: r,
        }
    }

    pub fn to_cubepos(&self) -> Pos {
        Pos::new(self.q, self.r - self.q, self.r)
    }

    pub fn fmt(&self) -> String {
        format!("{},{}", self.q, self.r)
    }
}

