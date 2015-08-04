/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use std::path::Path;

use rustty::{Event, Cell, CellAccessor};
use rustty::ui::Painter;

pub use civng::hexpos::{Pos, Direction};
use civng::map::TerrainMap;
use civng::screen::{Screen, DisplayOption};
use civng::civ5map::load_civ5map;

extern crate rustty;
extern crate civng;

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

struct Game {
    screen: Screen,
    map: TerrainMap,
    unitpos: Pos,
    scrollmode: bool, // tmp hack
}

impl Game {
    fn moveunit(&mut self, direction: Direction) {
        let newpos = self.unitpos.neighbor(direction);
        // Special case for impoassable startup position. We can move everywhere.
        if !self.map.get_terrain(self.unitpos).is_passable() || self.map.get_terrain(newpos).is_passable() {
            self.unitpos = newpos;
        }
        self.update_details();
    }

    fn update_details(&mut self) {
        self.screen.details_window.clear(Cell::default());
        let terrain = self.map.get_terrain(self.unitpos);
        self.screen.details_window.printline(2, 1, terrain.name());
        if self.scrollmode {
            self.screen.details_window.printline(2, 2, "Scroll Mode");
        }
        self.screen.details_window.draw_box();
    }
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
            if k == 's' {
                game.scrollmode = !game.scrollmode;
                game.update_details();
            }
            match direction_for_key(k) {
                Some(d) => {
                    if game.scrollmode {
                        game.screen.scroll(Pos::origin().neighbor(d));
                    }
                    else {
                        game.moveunit(d);
                    }
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
        scrollmode: false,
    };
    game.update_details();
    loop {
        game.screen.draw(&game.map, game.unitpos);
        if !handle_events(&mut game) {
            break;
        }
    }
}

