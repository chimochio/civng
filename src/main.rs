use std::path::Path;
use rustty::{Event};
use hexpos::{Pos, Direction};
use map::TerrainMap;
use screen::Screen;

extern crate num;
extern crate rustty;

mod hexpos;
mod map;
mod screen;

fn direction_for_key(key: char) -> Option<Direction> {
    match key {
        '8' => Some(Direction::North),
        '9' => Some(Direction::NorthEast),
        '3' => Some(Direction::SouthEast),
        '2' => Some(Direction::South),
        '1' => Some(Direction::SouthWest),
        '7' => Some(Direction::NorthWest),
        _ => None,
    }
}

fn moveunit(pos: Pos, direction: Direction, map: &TerrainMap) -> Pos {
    let newpos = pos.neighbor(direction);
    if map.get_terrain(newpos).is_passable() { newpos } else { pos }
}

fn main() {
    // top left corner is 0, 0 in axial. arrays below are rows of columns (axial pos).
    // true == wall. outside map == wall
    let map = TerrainMap::fromfile(Path::new("resources/simplemap.txt"));
    let mut unitpos = Pos::new(0, 0, 0);
    loop {
        let mut screen = Screen::new();
        screen.draw(&map, unitpos);
        match screen.term.get_event(-1) {
            Ok(Some(Event::Key(k))) => {
                if k == 'q' {
                    break;
                }
                match direction_for_key(k) {
                    Some(d) => {
                        unitpos = moveunit(unitpos, d, &map);
                    },
                    None => {},
                };
            },
            _ => { break; },
        }
    }
}

