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

/// Make `from` attack `to` and returns a CombatResult.
///
/// An attack *never* result in both units being dead.
pub fn attack(from: &Unit, to: &Unit) -> CombatResult {
    let mut rng = rand::thread_rng();
    let defender_is_weak = from.strength() > to.strength();
    let (strong, weak) = if defender_is_weak { (from, to) } else { (to, from) };
    let r = strong.strength() as f32 / weak.strength() as f32;
    let m = 0.5 + num::pow(r+3.0, 4) / 512.0;
    const BASE_MIN: f32 = 40.0;
    const BASE_SPREAD: f32 = 30.0;
    let weak_min = BASE_MIN * m;
    let weak_spread = BASE_SPREAD * m;
    let weak_rnd = Range::new(0.0, weak_spread).ind_sample(&mut rng);
    let mut dmg_to_weak = max((weak_min + weak_rnd).floor() as i32, 1);
    let strong_min = BASE_MIN / m;
    let strong_spread = BASE_SPREAD / m;
    let strong_rnd = Range::new(0.0, strong_spread).ind_sample(&mut rng);
    let mut dmg_to_strong = max((strong_min + strong_rnd).floor() as i32, 1);
    let weak_hp = weak.hp() as i32 - dmg_to_weak;
    let strong_hp = strong.hp() as i32  - dmg_to_strong;
    if weak_hp < 0 && strong_hp < 0 {
        // Only one unit can die. Revive the "less dead" one.
        if weak_hp > strong_hp {
            dmg_to_weak = weak.hp() as i32 - 1
        }
        else {
            dmg_to_strong = strong.hp() as i32 - 1
        }
    }
    let (dmg_to_from, dmg_to_to) = if defender_is_weak {
        (dmg_to_strong as u8, dmg_to_weak as u8)
    }
    else {
        (dmg_to_weak as u8, dmg_to_strong as u8)
    };
    CombatResult::new(from, to, dmg_to_from, dmg_to_to)
}

