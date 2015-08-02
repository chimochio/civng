/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use std::path::Path;

use rustty::{Event};

pub use civng::hexpos::{Pos, Direction};
use civng::map::TerrainMap;
use civng::screen::{Screen, DisplayOption};
use civng::civ5map::load_civ5map;

extern crate num;
extern crate rustty;
extern crate byteorder;
extern crate civng;

struct Game {
    screen: Screen,
    map: TerrainMap,
    unitpos: Pos,
}

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

/// Returns whether the mainloop should continue
fn handle_events(game: &mut Game) -> bool {
    match game.screen.term.get_event(-1) {
        Ok(Some(Event::Key(k))) => {
            if k == 'q' {
                return false;
            }
            if k == 'p' {
                game.screen.toggle_option(DisplayOption::PosMarkers);
            }
            match direction_for_key(k) {
                Some(d) => {
                    game.unitpos = moveunit(game.unitpos, d, &game.map);
                },
                None => {},
            };
        },
        _ => { return false; },
    }
    true
}

fn main() {
    let map = load_civ5map(Path::new("resources/pangea-duel.Civ5Map"));
    let mut game = Game {
        screen: Screen::new(),
        map: map,
        unitpos: Pos::new(0, 0, 0),
    };
    loop {
        game.screen.draw(&game.map, game.unitpos);
        if !handle_events(&mut game) {
            break;
        }
    }
}

