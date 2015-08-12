/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use hexpos::{Pos, Direction};
use unit::Unit;
use terrain::TerrainMap;

pub struct LiveMap {
    terrain: TerrainMap,
    units: Vec<Unit>,
}

impl LiveMap {
    pub fn new(terrain: TerrainMap) -> LiveMap {
        LiveMap {
            terrain: terrain,
            units: Vec::new(),
        }
    }

    pub fn terrain(&self) -> &TerrainMap {
        &self.terrain
    }

    pub fn units(&self) -> &Vec<Unit> {
        &self.units
    }

    pub fn create_unit(&mut self, pos: Pos) -> &mut Unit {
        let unit = Unit::new(pos);
        self.units.push(unit);
        let newlen = self.units.len();
        &mut self.units[newlen-1]
    }

    pub fn moveunit(&mut self, direction: Direction) -> bool {
        self.units[0].move_(direction, &self.terrain)
    }

    pub fn refresh(&mut self) {
        for unit in self.units.iter_mut() {
            unit.refresh();
        }
    }
}

