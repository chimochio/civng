// Copyright 2015 Virgil Dupras
//
// This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
// which should be included with this package. The terms are also available at
// http://www.gnu.org/licenses/gpl-3.0.html
//

use std::path::Path;

use civng::game::Game;
use civng::unit::{Unit, UnitType, Player};
use civng::hexpos::{Pos, OffsetPos};

extern crate rustty;
extern crate civng;

fn main() {
    let mut game = Game::new(Path::new("resources/pangea-duel.Civ5Map"));
    let unitpos = game.map().first_passable(Pos::origin());
    let _ = game.add_unit(Unit::new(UnitType::Melee, Player::Me, unitpos));
    let unitpos = game.map().first_passable(Pos::origin());
    let _ = game.add_unit(Unit::new(UnitType::Ranged, Player::Me, unitpos));
    let unitpos = game.map().first_passable(OffsetPos::new(4, 3).to_pos());
    let _ = game.add_unit(Unit::new(UnitType::Melee, Player::NotMe, unitpos));
    let unitpos = game.map().first_passable(OffsetPos::new(4, 3).to_pos());
    let _ = game.add_unit(Unit::new(UnitType::Melee, Player::NotMe, unitpos));
    game.new_turn();
    loop {
        game.draw();
        if !game.handle_events() {
            break;
        }
    }
}
