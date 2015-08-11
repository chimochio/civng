use hexpos::{Pos, Direction};
use map::TerrainMap;

pub struct Unit<'a> {
    map: &'a TerrainMap,
    pos: Pos,
    movements: u8,
}

impl<'a> Unit<'a> {
    pub fn new(map: &TerrainMap, pos: Pos) -> Unit {
        Unit {
            map: map,
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

    pub fn move_(&mut self, direction: Direction) -> bool {
        if self.movements == 0 {
            return false
        }
        let newpos = self.pos.neighbor(direction);
        if self.map.get_terrain(newpos).is_passable() {
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
