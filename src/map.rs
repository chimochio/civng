// Copyright 2015 Virgil Dupras
//
// This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
// which should be included with this package. The terms are also available at
// http://www.gnu.org/licenses/gpl-3.0.html
//

use std::collections::hash_map::{HashMap, Entry};

use hexpos::{Pos, PathWalker, PosPath};
use unit::{Unit, Units, UnitID, Player};
use terrain::{TerrainMap, Terrain};
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
        } else {
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
                return pos;
            }
        }
        panic!("No tile is passable!");
    }

    pub fn add_unit(&mut self, unit: Unit) {
        self.units.add_unit(unit)
    }

    fn get_terrain_modifier(&self, unit_id: UnitID) -> Option<Modifier> {
        let unit = self.units.get(unit_id);
        let terrain = self.terrain.get_terrain(unit.pos());
        let terrain_modifer_amount = terrain.defense_modifier();
        if terrain_modifer_amount != 0 {
            Some(Modifier::new(terrain_modifer_amount, ModifierType::Terrain))
        } else {
            None
        }
    }

    fn get_flanking_modifier(&self, against_id: UnitID) -> Option<Modifier> {
        let against = self.units.get(against_id);
        let mut flank_count = 0;
        let mut walker = PathWalker::new(against.pos(), 1);
        while let Some(p) = walker.next() {
            if let Some(uid) = self.units.unit_at_pos(p.to()) {
                if self.units.get(uid).owner() != against.owner() {
                    flank_count += 1;
                }
            }
        }
        if flank_count > 1 {
            Some(Modifier::new((flank_count - 1) * 10, ModifierType::Flanking))
        } else {
            None
        }
    }

    fn get_unit_modifiers(&self,
                          unit_id: UnitID,
                          against_id: UnitID,
                          defends: bool)
                          -> Vec<Modifier> {
        let mut result = Vec::new();
        if defends {
            if let Some(m) = self.get_terrain_modifier(unit_id) {
                result.push(m);
            }
        }
        if let Some(m) = self.get_flanking_modifier(against_id) {
            result.push(m);
        }
        result
    }

    pub fn moveunit_to(&mut self, unit_id: UnitID, pos: Pos) -> Option<CombatStats> {
        if let Some(path) = self.reachable_pos(unit_id).get(&pos).cloned() {
            let livepath = LivePath::new(&path, &self);
            if let Some(defender_id) = self.units.unit_at_pos(path.to()) {
                if path.steps() > 1 {
                    assert!(self.units.unit_at_pos(path.before_last().unwrap()).is_none());
                    self.moveunit_to(unit_id, path.before_last().unwrap());
                }
                let defender = self.units.get(defender_id);
                assert!(defender.owner() != Player::Me);
                let attacker = self.units.get(unit_id);
                let attacker_modifiers = self.get_unit_modifiers(attacker.id(),
                                                                 defender.id(),
                                                                 false);
                let defender_modifiers = self.get_unit_modifiers(defender.id(),
                                                                 attacker.id(),
                                                                 true);
                let combat_result = CombatStats::new(attacker,
                                                     attacker_modifiers,
                                                     defender,
                                                     defender_modifiers);
                return Some(combat_result);
            }
            let unit = self.units.get_mut(unit_id);
            let cost = if livepath.is_exhausting() {
                unit.movements()
            } else {
                livepath.cost()
            };
            unit.move_to(path.to(), cost);
        }
        None
    }

    pub fn attack(&mut self, combat_stats: &mut CombatStats) {
        self.units.attack(combat_stats);
    }

    pub fn refresh(&mut self) {
        self.units.refresh();
    }

    pub fn reachable_pos(&self, unit_id: UnitID) -> HashMap<Pos, PosPath> {
        let unit = self.units.get(unit_id);
        let mut result = HashMap::new();
        let mut walker = PathWalker::new(unit.pos(), unit.movements() as usize);
        while let Some(path) = walker.next() {
            let livepath = LivePath::new(&path, &self);
            if !livepath.could_be_reachable() {
                walker.backoff();
                continue;
            }
            let cost = livepath.cost();
            if livepath.is_reachable() {
                match result.entry(path.to()) {
                    Entry::Occupied(mut e) => {
                        // We replace the path only if the cost of the newer path is lower.
                        let oldcost = LivePath::new(e.get(), &self).cost();
                        if cost < oldcost {
                            e.insert(path.clone());
                        }
                    }
                    Entry::Vacant(e) => {
                        e.insert(path.clone());
                    }
                }
            }
            if cost >= unit.movements() {
                walker.backoff();
            }
        }
        result
    }
}

bitflags! {
    #[doc="Movement hindrances on a particular position on a live map,
        from the perspective of a player."]
    flags Hindrances: u8 {
        #[doc="A unit is on the cell"]
        const HINDRANCE_UNIT = 0b01,
        #[doc="The cell is affected by Zone of Control of an enemy unit"]
        const HINDRANCE_ZOC = 0b10,
    }
}

/// A `PosPath` along with terrain and unit information in that path.
pub struct LivePath {
    path: PosPath,
    terrain: Vec<Terrain>,
    hindrances: Vec<Hindrances>,
    mover: Option<Player>,
    target: Option<Player>,
}

impl LivePath {
    pub fn new(path: &PosPath, map: &LiveMap) -> LivePath {
        fn get_hindrances(map: &LiveMap, pos: Pos, mover: Option<Player>) -> Hindrances {
            let mut result = Hindrances::empty();
            if let Some(mover_owner) = mover {
                if let Some(u) = map.units().get_at_pos(pos) {
                    result.insert(HINDRANCE_UNIT);
                    if u.owner() != mover_owner {
                        result.insert(HINDRANCE_ZOC);
                    }
                }
                for neighbor in pos.around().iter() {
                    if let Some(u) = map.units().get_at_pos(*neighbor) {
                        if u.owner() != mover_owner {
                            result.insert(HINDRANCE_ZOC);
                        }
                    }
                }
            }
            result
        }

        let stack = path.stack();
        assert!(!stack.is_empty());
        let mover = {
            match map.units().get_at_pos(*stack.first().unwrap()) {
                Some(u) => Some(u.owner()),
                None => None,
            }
        };
        let target = {
            match map.units().get_at_pos(*stack.last().unwrap()) {
                Some(u) => Some(u.owner()),
                None => None,
            }
        };
        let terrain = stack.iter().map(|pos| map.terrain().get_terrain(*pos)).collect();
        let hindrances = stack.iter().map(|pos| get_hindrances(map, *pos, mover)).collect();
        LivePath {
            path: path.clone(),
            terrain: terrain,
            hindrances: hindrances,
            mover: mover,
            target: target,
        }
    }

    fn moves_through_zoc(&self, including_last_index: bool) -> bool {
        // Check for ZOC effect. A unit moving from a cell being in a ZOC to another cell being in
        // a ZOC cannot go any further.
        let mut last_index = self.hindrances.len();
        if !including_last_index {
            last_index -= 1;
        }
        let mut was_zoc = false;
        for hindrance in self.hindrances[0..last_index].iter() {
            if hindrance.contains(HINDRANCE_ZOC) {
                if was_zoc {
                    return true;
                } else {
                    was_zoc = true;
                }
            } else {
                was_zoc = false;
            }
        }
        false
    }

    pub fn is_attack(&self) -> bool {
        if let Some(mover_owner) = self.mover {
            match self.target {
                Some(p) => p != mover_owner,
                None => false,
            }
        } else {
            false
        }
    }

    /// Whether this path could ever become reachable by adding steps.
    pub fn could_be_reachable(&self) -> bool {
        if self.mover.is_none() {
            false
        } else if self.terrain.iter().any(|t| !t.is_passable()) {
            false
        } else {
            !self.moves_through_zoc(false)
        }
    }

    /// Whether this path is reachable by `self.mover()`.
    pub fn is_reachable(&self) -> bool {
        if !self.could_be_reachable() {
            false
        } else if self.path.steps() == 0 {
            false
        } else {
            let last_pos_hindrance = self.hindrances.last().unwrap();
            !last_pos_hindrance.contains(HINDRANCE_UNIT) || self.is_attack()
        }
    }

    /// Cost in movements required to move through that path.
    pub fn cost(&self) -> u8 {
        self.terrain[1..].iter().fold(0, |acc, &t| acc + t.movement_cost())
    }

    /// Whether the movement exhaust all movements of the mover, regardless of terrain costs.
    ///
    /// This happens when we move through an enemy ZOC.
    pub fn is_exhausting(&self) -> bool {
        self.moves_through_zoc(true)
    }
}
