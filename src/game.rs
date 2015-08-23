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
        &self.map.units().get(self.active_unit_index.unwrap_or(0))
    }

    fn cycle_active_unit(&mut self) {
        let active_index = match self.active_unit_index {
            Some(active_index) => active_index,
            None => self.map.units().max_id() + 1,
        };
        self.active_unit_index = self.map.units().next_active_unit(active_index);
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
                format!("MV {} / HP {}", unit.movements(), unit.hp()),
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

    pub fn add_unit(&mut self, unit: Unit) -> &mut Unit {
        self.map.add_unit(unit)
    }

    pub fn moveunit(&mut self, direction: Direction) {
        if self.active_unit_index.is_none() {
            return;
        }
        self.map.moveunit(self.active_unit_index.unwrap(), direction);
        if self.active_unit().is_exhausted() {
            self.cycle_active_unit();
        }
        self.update_details();
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

