use hexpos::{Pos, Direction};
use terrain::TerrainMap;

pub struct Unit {
    name: String,
    pos: Pos,
    movements: u8,
}

impl Unit {
    pub fn new(name: &str, pos: Pos) -> Unit {
        Unit {
            name: name.to_owned(),
            pos: pos,
            movements: 0,
        }
    }

    pub fn pos(&self) -> Pos {
        self.pos
    }

    pub fn movements(&self) -> u8 {
        self.movements
    }

    pub fn name(&self) -> &str {
        &self.name[..]
    }

    pub fn map_symbol(&self) -> char {
        self.name.chars().next().unwrap()
    }

    pub fn is_exhausted(&self) -> bool {
        self.movements == 0
    }

    pub fn move_(&mut self, direction: Direction, map: &TerrainMap) -> bool {
        if self.movements == 0 {
            return false
        }
        let newpos = self.pos.neighbor(direction);
        if map.get_terrain(newpos).is_passable() {
            self.pos = newpos;
            self.movements -= 1;
            return true
        }
        false
    }

    pub fn refresh(&mut self) {
        self.movements = 2;
    }
}
