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

    pub fn is_pos_passable(&self, pos: Pos) -> bool {
        if !self.terrain.get_terrain(pos).is_passable() {
            false
        }
        else {
            self.units.iter().all(|u| u.pos() != pos)
        }
    }

    /// Returns the first passable tile after `from`.
    ///
    /// Iterates all tiles from left to right, from the position `pos`. As soon as a tile is
    /// passable (terrain-wise and unit-wise), we return its position.
    ///
    /// # Examples
    ///
    /// ```
    /// use civng::terrain::TerrainMap;
    /// use civng::map::LiveMap;
    /// use civng::hexpos::Pos;
    ///
    /// let map = LiveMap::new(TerrainMap::empty_map(2, 2));
    /// assert_eq!(map.first_passable(Pos::origin()), Pos::origin());
    /// ```
    pub fn first_passable(&self, from: Pos) -> Pos {
        for (pos, _) in self.terrain.tiles().skip_while(|&(p, _)| p != from) {
            if self.is_pos_passable(pos) {
                return pos
            }
        }
        panic!("No tile is passable!");
    }

    pub fn add_unit(&mut self, unit: Unit) -> &mut Unit {
        self.units.push(unit);
        let newlen = self.units.len();
        &mut self.units[newlen-1]
    }

    pub fn moveunit(&mut self, unit_id: usize, direction: Direction) -> bool {
        let newpos = {
            let unit = &self.units[unit_id];
            let newpos = unit.pos().neighbor(direction);
            if !self.is_pos_passable(newpos) {
                return false
            }
            newpos
        };
        let unit = &mut self.units[unit_id];
        let terrain = self.terrain.get_terrain(newpos);
        unit.move_(direction, terrain)
    }

    pub fn refresh(&mut self) {
        for unit in self.units.iter_mut() {
            unit.refresh();
        }
    }
}

