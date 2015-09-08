/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

//! Unit management logic.

use std::cmp::min;
use std::collections::{HashMap, HashSet};

use combat::CombatStats;
use hexpos::Pos;

pub type UnitID = usize;

#[derive(Clone, Copy, PartialEq)]
pub enum Player {
    Me,
    NotMe,
}

/// A unit on a map.
pub struct Unit {
    id: UnitID,
    /// Name of the unit
    name: String,
    /// Position on the map
    pos: Pos,
    /// Movement points left this turn
    movements: u8,
    strength: u8,
    hp: u8,
    /// Player the unit belongs to
    owner: Player,
}

impl Unit {
    pub fn new(name: &str, owner: Player, pos: Pos) -> Unit {
        Unit {
            id: 0, // set in Units::add_unit()
            name: name.to_owned(),
            pos: pos,
            movements: 0,
            strength: 8,
            hp: 100,
            owner: owner,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn pos(&self) -> Pos {
        self.pos
    }

    pub fn movements(&self) -> u8 {
        self.movements
    }

    pub fn strength(&self) -> u8 {
        self.strength
    }

    pub fn hp(&self) -> u8 {
        self.hp
    }

    pub fn name(&self) -> &str {
        &self.name[..]
    }

    pub fn owner(&self) -> Player {
        self.owner
    }


    /// One letter symbol to represent the unit with on the map.
    ///
    /// For now, it's the first letter of the name.
    ///
    /// # Examples
    ///
    /// ```
    /// use civng::unit::{Unit, Player};
    /// use civng::hexpos::Pos;
    ///
    /// assert_eq!(Unit::new("Vincent", Player::Me, Pos::origin()).map_symbol(), 'V');
    /// ```
    pub fn map_symbol(&self) -> char {
        self.name.chars().next().unwrap()
    }

    /// Whether the unit as exhausted all movement points this turn.
    pub fn is_exhausted(&self) -> bool {
        self.movements == 0
    }

    pub fn is_dead(&self) -> bool {
        self.hp == 0
    }

    /// Move `self` in the position `target`.
    ///
    /// `cost` is the movement cost of the move, which will be subtracted of the unit's movements.
    ///
    /// # Examples
    ///
    /// ```
    /// use civng::unit::{Unit, Player};
    /// use civng::hexpos::{Pos, Direction};
    /// use civng::terrain::Terrain;
    ///
    /// let mut unit = Unit::new("Jules", Player::Me, Pos::origin());
    /// unit.refresh();
    /// let newpos = Pos::origin().neighbor(Direction::South);
    /// unit.move_to(newpos, 1);
    /// assert_eq!(unit.movements(), 1);
    /// assert_eq!(unit.pos(), newpos);
    /// ```
    pub fn move_to(&mut self, target: Pos, cost: u8) {
        self.pos = target;
        self.movements -= min(self.movements, cost);
    }

    /// Makes the unit fresh for a new turn.
    ///
    /// That is, regenerates its movement points.
    pub fn refresh(&mut self) {
        self.movements = 2;
    }

}

pub struct Units {
    maxid: UnitID,
    units: HashMap<UnitID, Unit>,
}

impl Units {
    pub fn new() -> Units {
        Units {
            maxid: 0,
            units: HashMap::new(),
        }
    }

    pub fn all_units<'a>(&'a self) -> Box<Iterator<Item=&'a Unit> + 'a> {
        Box::new(self.units.values().filter(|u| !u.is_dead()))
    }

    pub fn my_units<'a>(&'a self) -> Box<Iterator<Item=&'a Unit> + 'a>{
        Box::new(self.all_units().filter(|u| u.owner() == Player::Me))
    }

    pub fn add_unit(&mut self, mut unit: Unit) {
        self.maxid += 1;
        unit.id = self.maxid;
        self.units.insert(unit.id, unit);
    }

    pub fn attack(&mut self, combat_stats: &mut CombatStats) {
        let attacker_id = combat_stats.attacker_id;
        let defender_id = combat_stats.defender_id;
        combat_stats.roll();
        let defender_pos = {
            let defender = self.get_mut(defender_id);
            defender.hp = combat_stats.defender_remaining_hp();
            defender.pos
        };
        {
            let attacker = self.get_mut(attacker_id);
            attacker.hp = combat_stats.attacker_remaining_hp();
            attacker.movements = 0;
            if combat_stats.defender_remaining_hp() == 0 {
                attacker.pos = defender_pos;
            }
        }
    }

    pub fn max_id(&self) -> UnitID {
        self.maxid
    }

    /// Returns the next unit that should be activated after `after_id`.
    ///
    /// That unit is the smallest non-exhausted unit after the `after_id` unit. If it doesn't
    /// exist, it's the smallest non-exhausted id. Othewise, we return `None`.
    pub fn next_active_unit(&self, after_id: UnitID) -> Option<UnitID> {
        let mut result_before = None;
        let mut result_after = None;
        for unit in self.my_units() {
            if !unit.is_exhausted() {
                if unit.id() > after_id {
                    if result_after.is_none() || result_after.unwrap() > unit.id() {
                        result_after = Some(unit.id());
                    }
                }
                else {
                    if result_before.is_none() || result_before.unwrap() > unit.id() {
                        result_before = Some(unit.id());
                    }
                }
            }
        }
        result_after.or(result_before)
    }

    pub fn unit_at_pos(&self, pos: Pos) -> Option<UnitID> {
        for u in self.all_units() {
            if u.pos() == pos {
                return Some(u.id());
            }
        }
        None
    }

    /// Refreshes all units for a new turn and purges dead units from memory.
    pub fn refresh(&mut self) {
        let mut dead_unitids = HashSet::<UnitID>::new();
        for (_, unit) in self.units.iter_mut() {
            if !unit.is_dead() {
                unit.refresh();
            }
            else {
                dead_unitids.insert(unit.id());
            }
        }
        for unit_id in dead_unitids {
            self.units.remove(&unit_id);
        }
    }

    pub fn get(&self, unit_id: UnitID) -> &Unit {
        self.units.get(&unit_id).unwrap()
    }

    pub fn get_mut(&mut self, unit_id: UnitID) -> &mut Unit {
        self.units.get_mut(&unit_id).unwrap()
    }

    pub fn get_at_pos(&self, pos: Pos) -> Option<&Unit> {
        self.unit_at_pos(pos).map(|uid| self.get(uid))
    }
}

