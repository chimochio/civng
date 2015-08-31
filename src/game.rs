/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use std::path::Path;

use rustty::{Event, CellAccessor};
use rustty::ui::{Dialog, DialogResult};

use hexpos::{Pos, Direction};
use unit::{Unit, UnitID};
use screen::{Screen, DisplayOption};
use civ5map::load_civ5map;
use map::LiveMap;
use combat::CombatStats;
use combat_result_window::create_combat_result_dialog;
use combat_confirm_dialog::create_combat_confirm_dialog;

#[derive(Clone)]
enum MainloopState {
    Normal,
    CombatConfirm(CombatStats),
    MessageDialog,
}

/// Mode under which the game interprets movement keypresses.
#[derive(PartialEq)]
enum MovementMode {
    /// We move the active unit.
    Normal,
    /// We scroll the map.
    Scroll,
    /// We move the selected pos in the reachable range of the the active unit.
    Move,
}

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
    state: MainloopState,
    movemode: MovementMode,
    screen: Screen,
    map: LiveMap,
    turn: u16,
    active_unit_index: Option<UnitID>,
    /// Selected position when in Move mode.
    selected_pos: Pos,
    current_dialog: Option<Dialog>,
}

impl Game {
    pub fn new(map_path: &Path) -> Game {
        Game {
            state: MainloopState::Normal,
            movemode: MovementMode::Normal,
            screen: Screen::new(),
            map: {
                let terrainmap = load_civ5map(map_path);
                LiveMap::new(terrainmap)
            },
            turn: 0,
            active_unit_index: None,
            selected_pos: Pos::origin(),
            current_dialog: None,
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
        self.selected_pos = unitpos;
    }

    fn update_details(&mut self) {
        let movemode = match self.movemode {
            MovementMode::Scroll => "Scroll Mode",
            MovementMode::Move => "Move Mode",
            _ => "",
        };
        self.screen.details_window.update(
            self.active_unit_index, &self.map, self.turn, movemode
        );
    }

    pub fn map(&self) -> &LiveMap {
        &self.map
    }

    pub fn add_unit(&mut self, unit: Unit) -> &mut Unit {
        self.map.add_unit(unit)
    }

    // Code duplication with `moveunit()` is temporary.
    pub fn moveunit_to(&mut self, target: Pos) -> bool {
        if self.active_unit_index.is_none() {
            return false;
        }
        let result = self.map.moveunit_to(self.active_unit_index.unwrap(), target);
        self.selected_pos = self.active_unit().pos();
        if self.active_unit().is_exhausted() {
            self.cycle_active_unit();
        }
        self.update_details();
        result
    }

    pub fn moveunit(&mut self, direction: Direction) -> Option<CombatStats> {
        if self.active_unit_index.is_none() {
            return None;
        }
        let result = self.map.moveunit(self.active_unit_index.unwrap(), direction);
        self.selected_pos = self.active_unit().pos();
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
        let popup = match self.current_dialog {
            Some(ref mut d) => Some(d.window_mut()),
            None => None,
        };
        let selected_pos = match self.movemode {
            MovementMode::Move => Some(self.selected_pos),
            _ => None,
        };
        self.screen.draw(&self.map, self.active_unit_index, selected_pos, popup);
    }

    /// Returns whether the keypress was handled by the current dialog.
    ///
    /// If it was, then it shouldn't be handled by the normal loop.
    fn handle_messagedialog_keypress(&mut self, key: char) {
        assert!(self.current_dialog.is_some());
        let r = self.current_dialog.as_ref().unwrap().result_for_key(key);
        match r {
            Some(DialogResult::Ok) => {
                self.state = MainloopState::Normal;
                self.current_dialog = None;
                self.cycle_active_unit();
                self.update_details();
            },
            _ => {},
        }
    }

    fn handle_combatconfirm_keypress(&mut self, key: char, combat_stats: &mut CombatStats) {
        assert!(self.current_dialog.is_some());
        let r = self.current_dialog.as_ref().unwrap().result_for_key(key);
        match r {
            Some(DialogResult::Ok) => {
                self.map.attack(combat_stats);
                self.update_details();
                self.current_dialog = Some(create_combat_result_dialog(combat_stats));
                self.state = MainloopState::MessageDialog;
            },
            Some(DialogResult::Cancel) => {
                self.state = MainloopState::Normal;
                self.current_dialog = None;
            },
            _ => {},
        }
    }

    /// Returns whether the mainloop should continue
    fn handle_normal_keypress(&mut self, key: char) -> bool {
        match key {
            'Q' => { return false; },
            'P' => {
                self.screen.toggle_option(DisplayOption::PosMarkers);
            },
            'S' => {
                self.movemode = if self.movemode == MovementMode::Scroll { MovementMode::Normal } else { MovementMode::Scroll };
                self.update_details();
            },
            'm' => {
                match self.movemode {
                    MovementMode::Move => {
                        self.movemode = MovementMode::Normal;
                        self.selected_pos = self.active_unit().pos();
                    },
                    _ => {
                        self.movemode = MovementMode::Move;
                    },
                }
                self.update_details();
            },
            '\r' => {
                match self.movemode {
                    MovementMode::Move => {
                        let target = self.selected_pos;
                        self.moveunit_to(target);
                        self.movemode = MovementMode::Normal;
                        self.update_details();
                    },
                    _ => { self.new_turn(); },
                }
            },
            '.' => {
                self.cycle_active_unit();
                self.update_details();
                self.draw()
            },
            k => if let Some(d) = direction_for_key(k) {
                match self.movemode {
                    MovementMode::Normal => {
                        if let Some(ref combat_result) = self.moveunit(d) {
                            self.current_dialog = Some(create_combat_confirm_dialog(combat_result));
                            self.state = MainloopState::CombatConfirm(combat_result.clone());
                        }
                    },
                    MovementMode::Scroll => {
                        self.screen.scroll(Pos::origin().neighbor(d));
                    },
                    MovementMode::Move => {
                        self.selected_pos = self.selected_pos.neighbor(d);
                    },
                }
            },
        }
        true
    }

    /// Returns whether the mainloop should continue
    pub fn handle_events(&mut self) -> bool {
        match self.screen.term.get_event(-1) {
            Ok(Some(Event::Key(k))) => {
                match self.state.clone() {
                    MainloopState::Normal => {
                        if !self.handle_normal_keypress(k) {
                            return false;
                        }
                    },
                    MainloopState::MessageDialog => { self.handle_messagedialog_keypress(k); },
                    MainloopState::CombatConfirm(mut c) => { self.handle_combatconfirm_keypress(k, &mut c); },
                }
            },
            _ => { return false; },
        }
        true
    }
}

