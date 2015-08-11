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
use civng::unit::Unit;
use civng::screen::{Screen, DisplayOption};
use civng::civ5map::load_civ5map;

extern crate rustty;
extern crate civng;

fn direction_for_key(key: char) -> Option<Direction> {
    match key {
        'w' => Some(Direction::North),
        'e' => Some(Direction::NorthEast),
        'd' => Some(Direction::SouthEast),
        's' => Some(Direction::South),
        'a' => Some(Direction::SouthWest),
        'q' => Some(Direction::NorthWest),
        _ => None,
    }
}

struct Game<'a> {
    screen: Screen,
    map: &'a TerrainMap,
    unit: Unit<'a>,
    scrollmode: bool, // tmp hack
    turn: u16,
}

impl<'a> Game<'a> {
    fn moveunit(&mut self, direction: Direction) {
        if self.unit.move_(direction) {
            self.update_details();
        }
    }

    fn update_details(&mut self) {
        let dt = &mut self.screen.details_window;
        dt.clear(Cell::default());
        let terrain = self.map.get_terrain(self.unit.pos());
        dt.printline(2, 1, terrain.name());
        dt.printline(2, 2, &format!("Moves {}/2", self.unit.movements())[..]);
        dt.printline(2, 3, &format!("Turn {}", self.turn)[..]);
        if self.scrollmode {
            dt.printline(2, 4, "Scroll Mode");
        }
        dt.draw_box();
    }

    fn new_turn(&mut self) {
        self.turn += 1;
        self.unit.refresh();
        self.update_details()
    }
}

/// Returns whether the mainloop should continue
fn handle_events(game: &mut Game) -> bool {
    match game.screen.term.get_event(-1) {
        Ok(Some(Event::Key(k))) => {
            match k {
                'Q' => { return false; },
                'P' => {
                    game.screen.toggle_option(DisplayOption::PosMarkers);
                },
                'S' => {
                    game.scrollmode = !game.scrollmode;
                    game.update_details();
                },
                '\r' => {
                    game.new_turn();
                },
                k => match direction_for_key(k) {
                    Some(d) => {
                        if game.scrollmode {
                            game.screen.scroll(Pos::origin().neighbor(d));
                        }
                        else {
                            game.moveunit(d);
                        }
                    },
                    None => {},
                },
            }
        },
        _ => { return false; },
    }
    true
}

fn main() {
    let map = load_civ5map(Path::new("resources/pangea-duel.Civ5Map"));
    let unitpos = map.first_passable();
    let unit = Unit::new(&map, unitpos);
    let mut game = Game {
        screen: Screen::new(),
        map: &map,
        unit: unit,
        scrollmode: false,
        turn: 0,
    };
    game.new_turn();
    loop {
        game.screen.draw(&game.map, game.unit.pos());
        if !handle_events(&mut game) {
            break;
        }
    }
}

