/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use std::path::Path;

use rustty::{Event, Cell, CellAccessor};
use rustty::ui::Painter;

use hexpos::{Pos, Direction};
use unit::Unit;
use screen::{Screen, DisplayOption};
use civ5map::load_civ5map;
use map::LiveMap;

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

pub struct Game {
    screen: Screen,
    map: LiveMap,
    scrollmode: bool, // tmp hack
    turn: u16,
}

impl Game {
    pub fn new(map_path: &Path) -> Game {
        Game {
            screen: Screen::new(),
            map: {
                let terrainmap = load_civ5map(map_path);
                LiveMap::new(terrainmap)
            },
            scrollmode: false,
            turn: 0,
        }
    }

    pub fn unit(&self) -> &Unit {
        &self.map.units()[0]
    }

    pub fn map(&self) -> &LiveMap {
        &self.map
    }

    pub fn create_unit(&mut self, name: &str, pos: Pos) -> &mut Unit {
        self.map.create_unit(name, pos)
    }
    pub fn moveunit(&mut self, direction: Direction) {
        if self.map.moveunit(direction) {
            self.update_details();
        }
    }

    pub fn update_details(&mut self) {
        let lines = {
            let unit = self.unit();
            let terrain = self.map.terrain().get_terrain(unit.pos());
            [
                unit.name().to_owned(),
                format!("Moves {}/2", unit.movements()),
                terrain.name().to_owned(),
                format!("Turn {}", self.turn),
                (if self.scrollmode { "Scroll Mode" } else { "" }).to_owned(),
            ]
        };
        let dt = &mut self.screen.details_window;
        dt.clear(Cell::default());
        for (index, line) in lines.iter().enumerate() {
            dt.printline(2, index+1, line);
        }
        dt.draw_box();
    }

    pub fn new_turn(&mut self) {
        self.turn += 1;
        self.map.refresh();
        self.update_details()
    }

    pub fn draw(&mut self) {
        let screen = &mut self.screen;
        let unit = &self.map.units()[0];
        screen.draw(self.map.terrain(), unit);
    }

    /// Returns whether the mainloop should continue
    pub fn handle_events(&mut self) -> bool {
        match self.screen.term.get_event(-1) {
            Ok(Some(Event::Key(k))) => {
                match k {
                    'Q' => { return false; },
                    'P' => {
                        self.screen.toggle_option(DisplayOption::PosMarkers);
                    },
                    'S' => {
                        self.scrollmode = !self.scrollmode;
                        self.update_details();
                    },
                    '\r' => {
                        self.new_turn();
                    },
                    k => match direction_for_key(k) {
                        Some(d) => {
                            if self.scrollmode {
                                self.screen.scroll(Pos::origin().neighbor(d));
                            }
                            else {
                                self.moveunit(d);
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
}

