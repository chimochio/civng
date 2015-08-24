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

use unit::Unit;

// See http://forums.civfanatics.com/showthread.php?t=432238

pub struct CombatResult {
    pub attacker_name: String,
    pub defender_name: String,
    pub attacker_starting_hp: u8,
    pub defender_starting_hp: u8,
    pub dmg_to_attacker: u8,
    pub dmg_to_defender: u8,
}

impl CombatResult {
    pub fn new(attacker: &Unit, defender: &Unit, dmg_to_attacker: u8, dmg_to_defender: u8) -> CombatResult {
        CombatResult {
            attacker_name: attacker.name().to_owned(),
            defender_name: defender.name().to_owned(),
            attacker_starting_hp: attacker.hp(),
            defender_starting_hp: defender.hp(),
            dmg_to_attacker: dmg_to_attacker,
            dmg_to_defender: dmg_to_defender,
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
}

fn compute_dmg(source: &Unit, target: &Unit) -> u8 {
    let mut rng = rand::thread_rng();
    let target_is_weak = source.strength() > target.strength();
    let (strong, weak) = if target_is_weak { (source, target) } else { (target, source) };
    let r = strong.strength() as f32 / weak.strength() as f32;
    let mut m = 0.5 + num::pow(r+3.0, 4) / 512.0;
    if !target_is_weak {
        m = 1.0 / m;
    }
    const BASE_MIN: f32 = 40.0;
    const BASE_SPREAD: f32 = 30.0;
    let min = BASE_MIN * m;
    let spread = BASE_SPREAD * m;
    let rnd = Range::new(0.0, spread).ind_sample(&mut rng);
    let mut dmg_dealt = (min + rnd).floor();
    dmg_dealt = apply_penalty_for_damaged_unit(dmg_dealt, source.hp());
    max(dmg_dealt.floor() as i16, 1) as u8
}

fn apply_penalty_for_damaged_unit(dealt_dmg: f32, dealer_hp: u8) -> f32 {
    let penalty = ((100 - dealer_hp) / 20) as f32 * 0.1;
    dealt_dmg - (dealt_dmg * penalty)
}

/// Make `source` attack `target` and returns a CombatResult.
///
/// An attack *never* result in both units being dead.
pub fn attack(source: &Unit, target: &Unit) -> CombatResult {
    let mut dmg_to_target = compute_dmg(source, target);
    let mut dmg_to_source = compute_dmg(target, source);
    let target_hp = target.hp() as i16 - dmg_to_target as i16;
    let source_hp = source.hp() as i16 - dmg_to_source as i16;
    if target_hp < 0 && source_hp < 0 {
        // Only one unit can die. Revive the "less dead" one.
        if source_hp > target_hp {
            dmg_to_target = target.hp() - 1
        }
        else {
            dmg_to_source = source.hp() - 1
        }
    }
    CombatResult::new(source, target, dmg_to_source, dmg_to_target)
}

