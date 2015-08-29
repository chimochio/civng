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
    let mut d = Dialog::new(55, 12);
    {
        let w = d.window_mut();
        w.clear(Cell::default());
        let msg = "Expected results";
        let x = w.halign_line(msg, HorizontalAlign::Middle, 1);
        w.printline(x, 1, msg);
        let (amin, amax) = result.dmgrange_to_attacker();
        let admgfmt = format!("{}-{}", amin, amax);
        let (dmin, dmax) = result.dmgrange_to_defender();
        let ddmgfmt = format!("{}-{}", dmin, dmax);
        // temporary. later, we'll support more than one modifier...
        let defender_mods = if result.defender_modifiers.len() > 0 {
            result.defender_modifiers[0].description()
        }
        else {
            "None".to_owned()
        };
        let lines = [
            format!("Name          | {:<15} | {:<15}", result.attacker_name, result.defender_name),
            format!("Base Strength | {:<15} | {:<15}", result.attacker_base_strength, result.defender_base_strength),
            format!("Real Strength | {:<15} | {:<15}", result.attacker_strength(), result.defender_strength()),
            format!("HP            | {:<15} | {:<15}", result.attacker_starting_hp, result.defender_starting_hp),
            format!("Dmg incoming  | {:<15} | {:<15}", admgfmt, ddmgfmt),
            format!("Modifiers     | {:<15} | {:<15}", "None", defender_mods),
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

