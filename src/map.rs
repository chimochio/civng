/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use hexpos::{Pos, Direction};
use unit::{Unit, Units, Player};
use terrain::TerrainMap;
use combat::{CombatStats, Modifier, ModifierType};

pub struct LiveMap {
    terrain: TerrainMap,
    units: Units,
}

impl LiveMap {
    pub fn new(terrain: TerrainMap) -> LiveMap {
        LiveMap {
            terrain: terrain,
            units: Units::new(),
        }
    }

    pub fn terrain(&self) -> &TerrainMap {
        &self.terrain
    }

    pub fn units(&self) -> &Units {
        &self.units
    }

    pub fn is_pos_passable(&self, pos: Pos) -> bool {
        if !self.terrain.get_terrain(pos).is_passable() {
            false
        }
        else {
            self.units.unit_at_pos(pos) == None
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
        self.units.add_unit(unit)
    }

    pub fn moveunit(&mut self, unit_id: usize, direction: Direction) -> Option<CombatStats> {
        let newpos = self.units.get(unit_id).pos().neighbor(direction);
        let target_terrain = self.terrain.get_terrain(newpos);
        if !target_terrain.is_passable() {
            return None;
        }
        match self.units.unit_at_pos(newpos) {
            Some(uid) => {
                if self.units.get(uid).owner() == Player::Me {
                    return None
                }
                let attacker = self.units.get(unit_id);
                let defender = self.units.get(uid);
                let attacker_modifiers = Vec::new();
                let mut defender_modifiers = Vec::new();
                let terrain_modifer_amount = target_terrain.defense_modifier();
                if terrain_modifer_amount != 0 {
                    defender_modifiers.push(
                        Modifier::new(terrain_modifer_amount, ModifierType::Terrain)
                    );
                }
                let combat_result = CombatStats::new(attacker, attacker_modifiers, defender, defender_modifiers);
                return Some(combat_result);
            }
            None => (),
        }
        let unit = self.units.get_mut(unit_id);
        unit.move_(direction, target_terrain);
        None
    }

    pub fn attack(&mut self, combat_stats: &mut CombatStats) {
        self.units.attack(combat_stats);
    }

    pub fn refresh(&mut self) {
        self.units.refresh();
    }
}

