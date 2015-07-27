use num::integer::Integer;

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

/* "Cube"-type position. We simply call it Pos for conciseness because that's our "official" pos.
 */
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

    pub fn to_offset_pos(&self) -> OffsetPos {
        // Each x means +1x, -½y, -½z. y goes first.
        // Each y means -1y, +1z.
        let x = self.x;
        let y = self.z + self.x / 2;
        OffsetPos::new(x, y)
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

    pub fn to_pos(&self) -> Pos {
        Pos::new(self.q, self.r - self.q, self.r)
    }

    pub fn fmt(&self) -> String {
        format!("{},{}", self.q, self.r)
    }
}

/* "odd-q" type. Origin is top-left. (1, 0) is SouthEast of origin. (0, 1) is South.
 */
#[derive(Copy, Clone)]
pub struct OffsetPos {
    pub x: i32,
    pub y: i32,
}

impl OffsetPos {
    pub fn new(x: i32, y: i32) -> OffsetPos {
        OffsetPos {
            x: x,
            y: y,
        }
    }

    pub fn to_pos(&self) -> Pos {
        // Each x means +1x, -½y, -½z. y goes first.
        // Each y means -1y, +1z.
        let (halfx, remx) = self.x.div_rem(&2);
        let x = self.x;
        let y = (-self.y) + (-halfx - remx);
        let z = self.y + (-halfx);
        Pos::new(x, y, z)
    }

    pub fn fmt(&self) -> String {
        format!("{},{}", self.x, self.y)
    }
}
