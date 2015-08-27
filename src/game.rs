/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use std::path::Path;

use rustty::{Event, CellAccessor};
use rustty::ui::{DialogResult};

use hexpos::{Pos, Direction};
use unit::Unit;
use screen::{Screen, DisplayOption};
use civ5map::load_civ5map;
use map::LiveMap;
use combat::CombatResult;
use combat_result_window::create_combat_result_dialog;

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
        self.screen.details_window.update(self.active_unit_index, &self.map, self.turn, self.scrollmode);
    }

    pub fn map(&self) -> &LiveMap {
        &self.map
    }

    pub fn add_unit(&mut self, unit: Unit) -> &mut Unit {
        self.map.add_unit(unit)
    }

    pub fn moveunit(&mut self, direction: Direction) -> Option<CombatResult> {
        if self.active_unit_index.is_none() {
            return None;
        }
        let result = self.map.moveunit(self.active_unit_index.unwrap(), direction);
        if self.active_unit().is_exhausted() {
            self.cycle_active_unit();
        }
        self.update_details();
        result
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

    /// Returns whether the keypress was handled by the popup.
    ///
    /// If it was, then it shouldn't be handled by the normal loop.
    fn handle_popup_keypress(&mut self, key: char) -> bool {
        let has_popup = self.screen.popup_dialog.is_some();
        if has_popup {
            let r = self.screen.popup_dialog.as_ref().unwrap().result_for_key(key);
            match r {
                Some(DialogResult::Ok) => {
                    self.screen.popup_dialog = None;
                },
                _ => {},
            }
        }
        has_popup
    }
    /// Returns whether the mainloop should continue
    pub fn handle_events(&mut self) -> bool {
        match self.screen.term.get_event(-1) {
            Ok(Some(Event::Key(k))) => {
                if self.handle_popup_keypress(k) {
                    return true;
                }
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
                                match self.moveunit(d) {
                                    Some(ref combat_result) => {
                                        let combat_result_dialog = create_combat_result_dialog(combat_result);
                                        self.screen.popup_dialog = Some(combat_result_dialog);
                                    }
                                    None => (),
                                }
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

