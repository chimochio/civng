// Copyright 2016 Virgil Dupras
//
// This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
// which should be included with this package. The terms are also available at
// http://www.gnu.org/licenses/gpl-3.0.html
//

use std::cmp::max;

use num;
use rand;
use rand::distributions::{IndependentSample, Range};

use unit::{Unit, UnitID};

// See http://forums.civfanatics.com/showthread.php?t=432238

pub type DmgRange = (u8, u8);

#[derive(Clone)]
pub struct CombatStats {
    pub ranged: bool,
    pub attacker_id: UnitID,
    pub defender_id: UnitID,
    pub attacker_name: String,
    pub defender_name: String,
    pub attacker_base_strength: u8,
    pub defender_base_strength: u8,
    pub attacker_starting_hp: u8,
    pub defender_starting_hp: u8,
    pub dmg_to_attacker: u8,
    pub dmg_to_defender: u8,
    pub attacker_modifiers: Vec<Modifier>,
    pub defender_modifiers: Vec<Modifier>,
}

impl CombatStats {
    pub fn new(attacker: &Unit,
               attacker_modifiers: Vec<Modifier>,
               defender: &Unit,
               defender_modifiers: Vec<Modifier>)
               -> CombatStats {
        let atype = attacker.type_();
        let dtype = defender.type_();
        let ranged = atype.ranged_strength() > 0;
        let (astrength, dstrength) = if ranged {
            (atype.ranged_strength(),
             max(dtype.ranged_strength(), dtype.strength()))
        } else {
            (atype.strength(), dtype.strength())
        };
        CombatStats {
            ranged: ranged,
            attacker_id: attacker.id(),
            defender_id: defender.id(),
            attacker_name: attacker.name().to_owned(),
            defender_name: defender.name().to_owned(),
            attacker_base_strength: astrength,
            defender_base_strength: dstrength,
            attacker_starting_hp: attacker.hp(),
            defender_starting_hp: defender.hp(),
            dmg_to_attacker: 0,
            dmg_to_defender: 0,
            attacker_modifiers: attacker_modifiers,
            defender_modifiers: defender_modifiers,
        }
    }

    pub fn attacker_strength(&self) -> f32 {
        apply_modifier(self.attacker_base_strength as f32,
                       self.attacker_modifiers_total())
    }

    pub fn defender_strength(&self) -> f32 {
        apply_modifier(self.defender_base_strength as f32,
                       self.defender_modifiers_total())
    }

    pub fn attacker_modifiers_total(&self) -> i16 {
        sum_modifiers(&self.attacker_modifiers)
    }

    pub fn defender_modifiers_total(&self) -> i16 {
        sum_modifiers(&self.defender_modifiers)
    }

    pub fn dmgrange_to_attacker(&self) -> DmgRange {
        if self.ranged {
            (0, 0)
        } else {
            compute_dmg_range(self.defender_strength(),
                              self.defender_starting_hp,
                              self.attacker_strength(),
                              self.ranged)
        }
    }

    pub fn dmgrange_to_defender(&self) -> DmgRange {
        compute_dmg_range(self.attacker_strength(),
                          self.attacker_starting_hp,
                          self.defender_strength(),
                          self.ranged)
    }

    pub fn attacker_remaining_hp(&self) -> u8 {
        if self.dmg_to_attacker > self.attacker_starting_hp {
            0
        } else {
            self.attacker_starting_hp - self.dmg_to_attacker
        }
    }

    pub fn defender_remaining_hp(&self) -> u8 {
        if self.dmg_to_defender > self.defender_starting_hp {
            0
        } else {
            self.defender_starting_hp - self.dmg_to_defender
        }
    }

    pub fn roll(&mut self) {
        let mut dmg_to_attacker = roll_dice(self.dmgrange_to_attacker());
        let mut dmg_to_defender = roll_dice(self.dmgrange_to_defender());
        let defender_hp = self.defender_starting_hp as i16 - dmg_to_defender as i16;
        let attacker_hp = self.attacker_starting_hp as i16 - dmg_to_attacker as i16;
        if defender_hp < 0 && attacker_hp < 0 {
            // Only one unit can die. Revive the "less dead" one.
            if attacker_hp > defender_hp {
                dmg_to_attacker = self.attacker_starting_hp - 1;
            } else {
                dmg_to_defender = self.defender_starting_hp - 1;
            }
        }
        self.dmg_to_attacker = dmg_to_attacker;
        self.dmg_to_defender = dmg_to_defender;
    }
}

#[derive(Clone, Copy)]
pub enum ModifierType {
    Terrain,
    Flanking,
}

impl ModifierType {
    pub fn description(&self) -> &str {
        match *self {
            ModifierType::Terrain => "Terrain",
            ModifierType::Flanking => "Flanking",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Modifier {
    amount: i8, // 20 == +20%
    modtype: ModifierType,
}

impl Modifier {
    pub fn new(amount: i8, modtype: ModifierType) -> Modifier {
        Modifier {
            amount: amount,
            modtype: modtype,
        }
    }

    pub fn amount(&self) -> i8 {
        self.amount
    }

    pub fn description(&self) -> String {
        format!("{:+}% {}", self.amount, self.modtype.description())
    }
}

fn sum_modifiers(modifiers: &Vec<Modifier>) -> i16 {
    modifiers.iter().fold(0, |acc, m| acc + m.amount as i16)
}

fn apply_modifier(strength: f32, modifier: i16) -> f32 {
    let fmodifier = 1.0 + (modifier as f32 / 100.0);
    strength * fmodifier
}

fn roll_dice(range: DmgRange) -> u8 {
    let mut rng = rand::thread_rng();
    let (min, max) = range;
    // max+1 because Range excludes high bound.
    Range::new(min, max + 1).ind_sample(&mut rng)
}

fn compute_dmg_range(source_strength: f32,
                     source_hp: u8,
                     target_strength: f32,
                     ranged: bool)
                     -> DmgRange {
    let target_is_weak = source_strength > target_strength;
    let (strong_strength, weak_strength) = if target_is_weak {
        (source_strength, target_strength)
    } else {
        (target_strength, source_strength)
    };
    let r = strong_strength / weak_strength;
    let mut m = 0.5 + num::pow(r + 3.0, 4) / 512.0;
    if !target_is_weak {
        m = 1.0 / m;
    }
    let base_min: f32 = if ranged {
        20.0
    } else {
        40.0
    };
    const BASE_SPREAD: f32 = 30.0;
    let mut min = apply_penalty_for_damaged_unit(base_min * m, source_hp);
    if min < 1.0 {
        min = 1.0;
    }
    let spread = apply_penalty_for_damaged_unit(BASE_SPREAD * m, source_hp);
    (min.floor() as u8, (min + spread).floor() as u8)
}

fn apply_penalty_for_damaged_unit(dealt_dmg: f32, dealer_hp: u8) -> f32 {
    let penalty = ((100 - dealer_hp) / 20) as f32 * 0.1;
    dealt_dmg - (dealt_dmg * penalty)
}
