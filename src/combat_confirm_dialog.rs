/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use rustty::{CellAccessor, Cell};
use rustty::ui::{Painter, HorizontalAlign, Dialog, DialogResult};

use combat::CombatStats;

pub fn create_combat_confirm_dialog(result: &CombatStats) -> Dialog {
    let mut d = Dialog::new(35, 12);
    {
        let w = d.window_mut();
        w.clear(Cell::default());
        let msg = "Expected results";
        let x = w.halign_line(msg, HorizontalAlign::Middle, 1);
        w.printline(x, 1, msg);
        let (amin, amax) = result.dmgrange_to_attacker;
        let (dmin, dmax) = result.dmgrange_to_defender;
        let lines = [
            format!("Attacker: {}", result.attacker_name),
            format!("HP: {}", result.attacker_starting_hp),
            format!("Dmg incoming (min/max): {}/{}", amin, amax),
            format!("Defender: {}", result.defender_name),
            format!("HP: {}", result.defender_starting_hp),
            format!("Dmg incoming (min/max): {}/{}", dmin, dmax),
        ];
        for (i, s) in lines.iter().enumerate() {
            w.printline(2, 3+i, &s[..]);
        }
    }
    d.add_button("Attack", 'a', DialogResult::Ok);
    d.add_button("Withdraw", 'w', DialogResult::Cancel);
    d.draw_buttons();
    d.window_mut().draw_box();
    d
}

