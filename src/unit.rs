use hexpos::{Pos, Direction};
use map::TerrainMap;

pub struct Unit<'a> {
    map: &'a TerrainMap,
    pos: Pos,
}

impl<'a> Unit<'a> {
    pub fn new(map: &TerrainMap, pos: Pos) -> Unit {
        Unit {
            map: map,
            pos: pos,
        }
    }

    pub fn pos(&self) -> Pos {
        self.pos
    }

    pub fn move_(&mut self, direction: Direction) -> bool {
        let newpos = self.pos.neighbor(direction);
        if self.map.get_terrain(newpos).is_passable() {
            self.pos = newpos;
            return true
        }
        false
    }
}
