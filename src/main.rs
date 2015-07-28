/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use std::path::Path;
use rustty::{Event};
use hexpos::{Pos, Direction};
use map::TerrainMap;
use screen::Screen;
use civ5map::load_civ5map;

extern crate num;
extern crate rustty;
extern crate byteorder;

mod hexpos;
mod map;
mod screen;
mod civ5map;

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
    if !map.get_terrain(pos).is_passable() {
        // Special case for impoassable startup position. We can move everywhere.
        return newpos
    }
    if map.get_terrain(newpos).is_passable() { newpos } else { pos }
}

fn main() {
    let map = load_civ5map(Path::new("resources/pangea-duel.Civ5Map"));
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

