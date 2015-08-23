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

/// Make `from` attack `to` and returns `(new_from_hp, new_to_hp)`.
///
/// An attack *never* result in both units being dead.
pub fn attack(from: &Unit, to: &Unit) -> (u8, u8) {
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
    let dmg_to_weak = max((weak_min + weak_rnd).floor() as i32, 1);
    let strong_min = BASE_MIN / m;
    let strong_spread = BASE_SPREAD / m;
    let strong_rnd = Range::new(0.0, strong_spread).ind_sample(&mut rng);
    let dmg_to_strong = max((strong_min + strong_rnd).floor() as i32, 1);
    let mut weak_hp = weak.hp() as i32 - dmg_to_weak;
    let mut strong_hp = strong.hp() as i32  - dmg_to_strong;
    if weak_hp < 0 && strong_hp < 0 {
        // Only one unit can die. Revive the "less dead" one.
        if weak_hp > strong_hp {
            weak_hp = 1;
        }
        else {
            strong_hp = 1;
        }
    }
    weak_hp = max(weak_hp, 0);
    strong_hp = max(strong_hp, 0);
    if defender_is_weak {
        (strong_hp as u8, weak_hp as u8)
    }
    else {
        (weak_hp as u8, strong_hp as u8)
    }
}

