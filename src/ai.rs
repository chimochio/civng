/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use rand::{thread_rng, sample};

use hexpos::PosPath;
use unit::UnitID;
use map::{LivePath, LiveMap};

/// Make `unit_id` move in random directions until it exhausted its movements.
pub fn wander(unit_id: UnitID, map: &mut LiveMap) {
    let target_pos = {
        let target_cost = map.units().get(unit_id).movements();
        let reachable = map.reachable_pos(unit_id);
        if reachable.is_empty() {
            return;
        }
        let choices: Vec<&PosPath> = reachable.values().filter(
            |p| {
                let lp = LivePath::new(p, map);
                !lp.is_attack() && lp.cost() == target_cost
            }
        ).collect();
        let mut rng = thread_rng();
        sample(&mut rng, choices.iter(), 1).first().unwrap().to()
    };
    map.moveunit_to(unit_id, target_pos);
}

