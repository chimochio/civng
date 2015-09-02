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
use unit::{Unit};
use screen::{Screen, DisplayOption};
use civ5map::load_civ5map;
use map::LiveMap;
use combat::CombatStats;
use combat_result_window::create_combat_result_dialog;
use combat_confirm_dialog::create_combat_confirm_dialog;
use selection::Selection;

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
    selection: Selection,
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
            selection: Selection::new(),
            current_dialog: None,
        }
    }

    fn active_unit(&self) -> &Unit {
        &self.map.units().get(self.selection.unit_id.unwrap_or(0))
    }

    fn cycle_active_unit(&mut self) {
        let active_index = match self.selection.unit_id {
            Some(active_index) => active_index,
            None => self.map.units().max_id() + 1,
        };
        self.selection.unit_id = self.map.units().next_active_unit(active_index);
        let unitpos = self.active_unit().pos();
        let terrainmap = self.map.terrain();
        self.screen.center_on_pos(unitpos, terrainmap);
    }

    fn update_details(&mut self) {
        let movemode = match self.movemode {
            MovementMode::Scroll => "Scroll Mode",
            MovementMode::Move => "Move Mode",
            _ => "",
        };
        let selected_pos = self.selection.pos.unwrap_or({
            self.active_unit().pos()
        });
        self.screen.details_window.update(
            selected_pos, &self.map, self.turn, movemode
        );
    }

    pub fn map(&self) -> &LiveMap {
        &self.map
    }

    pub fn add_unit(&mut self, unit: Unit) -> &mut Unit {
        self.map.add_unit(unit)
    }

    // Code duplication with `moveunit()` is temporary.
    pub fn moveunit_to(&mut self, target: Pos) -> Option<CombatStats> {
        if self.selection.unit_id.is_none() {
            return None;
        }
        let result = self.map.moveunit_to(self.selection.unit_id.unwrap(), target);
        if self.active_unit().is_exhausted() {
            self.cycle_active_unit();
        }
        self.update_details();
        result
    }

    pub fn moveunit(&mut self, direction: Direction) -> Option<CombatStats> {
        if self.selection.unit_id.is_none() {
            return None;
        }
        let newpos = self.active_unit().pos().neighbor(direction);
        self.moveunit_to(newpos)
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
        self.screen.draw(&self.map, &self.selection, popup);
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
                        self.selection.pos = None;
                    },
                    _ => {
                        self.movemode = MovementMode::Move;
                        self.selection.pos = Some(self.active_unit().pos());
                    },
                }
                self.update_details();
            },
            '\r' => {
                match self.movemode {
                    MovementMode::Move => {
                        let target = self.selection.pos.unwrap();
                        if let Some(ref combat_result) = self.moveunit_to(target) {
                            self.current_dialog = Some(create_combat_confirm_dialog(combat_result));
                            self.state = MainloopState::CombatConfirm(combat_result.clone());
                        }
                        self.movemode = MovementMode::Normal;
                        self.selection.pos = None;
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
                        self.selection.pos = Some(self.selection.pos.unwrap().neighbor(d));
                        self.update_details();
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

