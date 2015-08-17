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
    active_unit_index: Option<usize>,
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
            active_unit_index: None,
        }
    }

    fn active_unit(&self) -> &Unit {
        &self.map.units()[self.active_unit_index.unwrap_or(0)]
    }

    fn cycle_active_unit(&mut self) {
        // We want to start at the current index and cycle from there, starting back at 0 when
        // we reach the end of the line. This is why we have this two-parts algo.
        let active_index = match self.active_unit_index {
            Some(active_index) => active_index,
            None => self.map.units().len(),
        };
        let first_half = self.map.units().iter().enumerate().take(active_index+1);
        let second_half = self.map.units().iter().enumerate().skip(active_index+1);
        match second_half.chain(first_half).skip_while(|&(_, u)| u.is_exhausted()).next() {
            Some((i, _)) => { self.active_unit_index = Some(i); },
            None => { self.active_unit_index = None; },
        }
        let unitpos = self.active_unit().pos();
        let terrainmap = self.map.terrain();
        self.screen.center_on_pos(unitpos, terrainmap);
    }

    fn update_details(&mut self) {
        let lines = {
            let unit = self.active_unit();
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

    pub fn map(&self) -> &LiveMap {
        &self.map
    }

    pub fn create_unit(&mut self, name: &str, pos: Pos) -> &mut Unit {
        self.map.create_unit(name, pos)
    }

    pub fn moveunit(&mut self, direction: Direction) {
        if self.active_unit_index.is_none() {
            return;
        }
        if self.map.moveunit(self.active_unit_index.unwrap(), direction) {
            if self.active_unit().is_exhausted() {
                self.cycle_active_unit();
            }
            self.update_details();
        }
    }

    pub fn new_turn(&mut self) {
        self.turn += 1;
        self.map.refresh();
        self.cycle_active_unit();
        self.update_details()
    }

    pub fn draw(&mut self) {
        self.screen.draw(&self.map, self.active_unit_index);
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
                    '.' => {
                        self.cycle_active_unit();
                        self.update_details();
                        self.draw()
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

