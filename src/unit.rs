/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use hexpos::{Pos, Direction};
use terrain::Terrain;

pub struct Unit {
    name: String,
    pos: Pos,
    movements: u8,
}

impl Unit {
    pub fn new(name: &str, pos: Pos) -> Unit {
        Unit {
            name: name.to_owned(),
            pos: pos,
            movements: 0,
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

    pub fn map_symbol(&self) -> char {
        self.name.chars().next().unwrap()
    }

    pub fn is_exhausted(&self) -> bool {
        self.movements == 0
    }

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

    pub fn refresh(&mut self) {
        self.movements = 2;
    }
}
