/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

//! Unit management logic.

use std::slice::Iter;

use combat;
use hexpos::{Pos, Direction};
use terrain::Terrain;

#[derive(Clone, Copy, PartialEq)]
pub enum Player {
    Me,
    NotMe,
}

/// A unit on a map.
pub struct Unit {
    id: usize,
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

    /// Move `self` in the specified `direction`.
    ///
    /// `terrain` being the terrain type at the specified destination. If the move is successful,
    /// returns `true` and deduct the appropriate movements points from the unit.
    ///
    /// If the unit doesn't have enough movement points or the terrain is impassable, no move take
    /// place and we return `false`.
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
    /// // We move alright!
    /// assert!(unit.move_(Direction::South, Terrain::Grassland));
    /// assert_eq!(unit.movements(), 1);
    /// assert_eq!(unit.pos(), Pos::vector(Direction::South));
    /// // Impassable
    /// assert!(!unit.move_(Direction::South, Terrain::Mountain));
    /// assert_eq!(unit.movements(), 1);
    /// assert_eq!(unit.pos(), Pos::vector(Direction::South));
    /// assert!(unit.move_(Direction::South, Terrain::Hill));
    /// // Costs 2 moves, but we don't go below 0
    /// assert_eq!(unit.movements(), 0);
    /// assert_eq!(unit.pos(), Pos::vector(Direction::South).amplify(2));
    /// // Out of moves
    /// assert!(!unit.move_(Direction::South, Terrain::Hill));
    /// ```
    pub fn move_(&mut self, direction: Direction, terrain: Terrain) -> bool {
        if self.movements == 0 || !terrain.is_passable() {
            return false
        }
        let newpos = self.pos.neighbor(direction);
        self.pos = newpos;
        let cost = terrain.movement_cost();
        if cost > self.movements {
            self.movements = 0;
        }
        else {
            self.movements -= cost;
        }
        true
    }

    /// Makes the unit fresh for a new turn.
    ///
    /// That is, regenerates its movement points.
    pub fn refresh(&mut self) {
        self.movements = 2;
    }
}

pub struct Units {
    units: Vec<Unit>,
}

impl Units {
    pub fn new() -> Units {
        Units {
            units: Vec::new(),
        }
    }

    pub fn all_units(&self) -> UnitsIterator {
        UnitsIterator::new(self.units.iter(), None)
    }

    pub fn my_units(&self) -> UnitsIterator {
        UnitsIterator::new(self.units.iter(), Some(Player::Me))
    }

    pub fn add_unit(&mut self, unit: Unit) -> &mut Unit {
        self.units.push(unit);
        let newlen = self.units.len();
        let result = &mut self.units[newlen-1];
        result.id = newlen - 1;
        result
    }

    pub fn attack(&mut self, from_id: usize, to_id: usize) {
        let (from_hp, to_hp) = combat::attack(&self.units[from_id], &self.units[to_id]);
        (&mut self.units[from_id]).hp = from_hp;
        (&mut self.units[from_id]).movements = 0;
        (&mut self.units[to_id]).hp = to_hp;
        if to_hp == 0 {
            (&mut self.units[from_id]).pos = (&mut self.units[to_id]).pos;
        }
    }

    pub fn max_id(&self) -> usize {
        self.units.len() - 1
    }

    pub fn next_active_unit(&self, after_id: usize) -> Option<usize> {
        // We want to start at the current index and cycle from there, starting back at 0 when
        // we reach the end of the line. This is why we have this two-parts algo.
        let second_half = self.my_units().skip_while(|u| u.id <= after_id);
        match second_half.filter(|u| !u.is_exhausted()).next() {
            Some(u) => Some(u.id),
            None => {
                match self.my_units().filter(|u| !u.is_exhausted()).next() {
                    Some(u) => Some(u.id),
                    None => None,
                }
            }
        }
    }

    pub fn unit_at_pos(&self, pos: Pos) -> Option<usize> {
        for u in self.all_units() {
            if u.pos() == pos {
                return Some(u.id());
            }
        }
        None
    }

    pub fn refresh(&mut self) {
        for unit in self.units.iter_mut() {
            unit.refresh();
        }
    }

    pub fn get(&self, unit_id: usize) -> &Unit {
        &self.units[unit_id]
    }

    pub fn get_mut(&mut self, unit_id: usize) -> &mut Unit {
        &mut self.units[unit_id]
    }

}

pub struct UnitsIterator<'a> {
    units_iter: Iter<'a, Unit>,
    owner: Option<Player>,
}

impl<'a> UnitsIterator<'a> {
    pub fn new(units_iter: Iter<Unit>, owner: Option<Player>) -> UnitsIterator{
        UnitsIterator {
            units_iter: units_iter,
            owner: owner,
        }
    }
}

impl<'a> Iterator for UnitsIterator<'a> {
    type Item = &'a Unit;

    fn next(&mut self) -> Option<&'a Unit> {
        while let Some(u) = self.units_iter.next() {
            if u.is_dead() {
                continue;
            }
            match self.owner {
                Some(o) => { if u.owner == o { return Some(u) }; },
                None => return Some(u),
            }
        }
        None
    }
}

