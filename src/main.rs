/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use std::path::Path;

use civng::game::Game;

extern crate rustty;
extern crate civng;

fn main() {
    let mut game = Game::new(Path::new("resources/pangea-duel.Civ5Map"));
    let unitpos = game.map().terrain().first_passable();
    let _ = game.create_unit("Lenny", unitpos);
    game.new_turn();
    loop {
        game.draw();
        if !game.handle_events() {
            break;
        }
    }
}

