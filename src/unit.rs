/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

//! Unit management logic.

use hexpos::{Pos, Direction};
use terrain::Terrain;

#[derive(Clone, Copy, PartialEq)]
pub enum Player {
    Me,
    NotMe,
}

/// A unit on a map.
pub struct Unit {
    /// Name of the unit
    name: String,
    /// Position on the map
    pos: Pos,
    /// Movement points left this turn
    movements: u8,
    /// Player the unit belongs to
    owner: Player,
}

impl Unit {
    pub fn new(name: &str, owner: Player, pos: Pos) -> Unit {
        Unit {
            name: name.to_owned(),
            pos: pos,
            movements: 0,
            owner: owner,
        }
    }

    pub fn pos(&self) -> Pos {
        self.pos
    }

    pub fn movements(&self) -> u8 {
        self.movements
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
