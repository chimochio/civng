// Copyright 2015 Virgil Dupras
//
// This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
// which should be included with this package. The terms are also available at
// http://www.gnu.org/licenses/gpl-3.0.html
//

use unit::UnitID;
use hexpos::Pos;

/// User's current selections among the game elements.
pub struct Selection {
    pub unit_id: Option<UnitID>,
    pub pos: Option<Pos>,
}

impl Selection {
    pub fn new() -> Selection {
        Selection {
            unit_id: None,
            pos: None,
        }
    }

    pub fn is_unit_active(&self, unit_id: UnitID) -> bool {
        match self.unit_id {
            Some(uid) => unit_id == uid,
            None => false,
        }
    }
}
