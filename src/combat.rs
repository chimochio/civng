/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use std::cmp::max;

use num;
use rand;
use rand::distributions::{IndependentSample, Range};

use unit::{Unit, UnitID};

// See http://forums.civfanatics.com/showthread.php?t=432238

pub type DmgRange = (u8, u8);

#[derive(Clone)]
pub struct CombatStats {
    pub attacker_id: UnitID,
    pub defender_id: UnitID,
    pub attacker_name: String,
    pub defender_name: String,
    pub attacker_starting_hp: u8,
    pub defender_starting_hp: u8,
    pub dmgrange_to_attacker: DmgRange,
    pub dmgrange_to_defender: DmgRange,
    pub dmg_to_attacker: u8,
    pub dmg_to_defender: u8,
}

impl CombatStats {
    pub fn new(attacker: &Unit, defender: &Unit) -> CombatStats {
        let dmgrange_to_defender = compute_dmg_range(attacker, defender);
        let dmgrange_to_attacker = compute_dmg_range(defender, attacker);
        CombatStats {
            attacker_id: attacker.id(),
            defender_id: defender.id(),
            attacker_name: attacker.name().to_owned(),
            defender_name: defender.name().to_owned(),
            attacker_starting_hp: attacker.hp(),
            defender_starting_hp: defender.hp(),
            dmgrange_to_attacker: dmgrange_to_attacker,
            dmgrange_to_defender: dmgrange_to_defender,
            dmg_to_attacker: 0,
            dmg_to_defender: 0,
        }
    }

    pub fn attacker_remaining_hp(&self) -> u8 {
        if self.dmg_to_attacker > self.attacker_starting_hp {
            0
        }
        else {
            self.attacker_starting_hp - self.dmg_to_attacker
        }
    }

    pub fn defender_remaining_hp(&self) -> u8 {
        if self.dmg_to_defender > self.defender_starting_hp {
            0
        }
        else {
            self.defender_starting_hp - self.dmg_to_defender
        }
    }

    pub fn roll(&mut self) {
        let mut dmg_to_attacker = roll_dice(self.dmgrange_to_attacker);
        let mut dmg_to_defender = roll_dice(self.dmgrange_to_defender);
        let defender_hp = self.defender_starting_hp as i16 - dmg_to_defender as i16;
        let attacker_hp = self.attacker_starting_hp as i16 - dmg_to_attacker as i16;
        if defender_hp < 0 && attacker_hp < 0 {
            // Only one unit can die. Revive the "less dead" one.
            if attacker_hp > defender_hp {
                dmg_to_attacker = self.attacker_starting_hp - 1;
            }
            else {
                dmg_to_defender = self.defender_starting_hp - 1;
            }
        }
        self.dmg_to_attacker = dmg_to_attacker;
        self.dmg_to_defender = dmg_to_defender;
    }
}

fn roll_dice(range: DmgRange) -> u8 {
    let mut rng = rand::thread_rng();
    let (min, max) = range;
    // max+1 because Range excludes high bound.
    Range::new(min, max+1).ind_sample(&mut rng)
}

fn compute_dmg_range(source: &Unit, target: &Unit) -> DmgRange {
    let target_is_weak = source.strength() > target.strength();
    let (strong, weak) = if target_is_weak { (source, target) } else { (target, source) };
    let r = strong.strength() as f32 / weak.strength() as f32;
    let mut m = 0.5 + num::pow(r+3.0, 4) / 512.0;
    if !target_is_weak {
        m = 1.0 / m;
    }
    const BASE_MIN: f32 = 40.0;
    const BASE_SPREAD: f32 = 30.0;
    let mut min = apply_penalty_for_damaged_unit(BASE_MIN * m, source.hp());
    if min < 1.0 {
        min = 1.0;
    }
    let spread = apply_penalty_for_damaged_unit(BASE_SPREAD * m, source.hp());
    (min.floor() as u8, (min + spread).floor() as u8)
}

fn apply_penalty_for_damaged_unit(dealt_dmg: f32, dealer_hp: u8) -> f32 {
    let penalty = ((100 - dealer_hp) / 20) as f32 * 0.1;
    dealt_dmg - (dealt_dmg * penalty)
}

