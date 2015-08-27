/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use rustty::{CellAccessor, Cell};
use rustty::ui::{Painter, HorizontalAlign, Dialog, DialogResult};

use combat::CombatResult;

pub fn create_combat_result_dialog(result: &CombatResult) -> Dialog {
    let mut d = Dialog::new(35, 12);
    {
        let w = d.window_mut();
        w.clear(Cell::default());
        let result_desc = if result.attacker_remaining_hp() == 0 {
            "Crushing Defeat"
        }
        else if result.defender_remaining_hp() == 0 {
            "Decisive Victory"
        }
        else if result.dmg_to_defender > result.dmg_to_attacker {
            "Victory"
        }
        else {
            "Defeat"
        };
        let x = w.halign_line(result_desc, HorizontalAlign::Middle, 1);
        w.printline(x, 1, result_desc);
        let lines = [
            format!("Attacker: {}", result.attacker_name),
            format!("Dmg received: {}", result.dmg_to_attacker),
            format!("Remaining HP: {}", result.attacker_remaining_hp()),
            format!("Defender: {}", result.defender_name),
            format!("Dmg received: {}", result.dmg_to_defender),
            format!("Remaining HP: {}", result.defender_remaining_hp()),
        ];
        for (i, s) in lines.iter().enumerate() {
            w.printline(2, 3+i, &s[..]);
        }
    }
    d.add_button("Ok", 'o', DialogResult::Ok);
    d.draw_buttons();
    d.window_mut().draw_box();
    d
}

