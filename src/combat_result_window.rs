/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use rustty::{CellAccessor, Cell};
use rustty::ui::{Painter, Widget, DrawArea, HorizontalAlign, VerticalAlign, create_button};

use dialog::Dialog;
use combat::CombatResult;

pub struct CombatResultWindow {
    window: Widget,
}

impl CombatResultWindow {
    pub fn new() -> CombatResultWindow {
        let window = Widget::new(35, 12);
        CombatResultWindow {
            window: window,
        }
    }

    pub fn draw_into(&self, cells: &mut CellAccessor) {
        self.window.draw_into(cells);
    }

    pub fn update(&mut self, result: &CombatResult) {
        self.window.clear(Cell::default());
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
        let x = self.window.halign_line(result_desc, HorizontalAlign::Middle, 1);
        self.window.printline(x, 1, result_desc);
        let lines = [
            format!("Attacker: {}", result.attacker_name),
            format!("Dmg received: {}", result.dmg_to_attacker),
            format!("Remaining HP: {}", result.attacker_remaining_hp()),
            format!("Defender: {}", result.defender_name),
            format!("Dmg received: {}", result.dmg_to_defender),
            format!("Remaining HP: {}", result.defender_remaining_hp()),
        ];
        for (i, s) in lines.iter().enumerate() {
            self.window.printline(2, 3+i, &s[..]);
        }
        let mut b = create_button("Ok", Some('o'));
        b.valign(&self.window, VerticalAlign::Bottom, 1);
        b.halign(&self.window, HorizontalAlign::Middle, 1);
        b.draw_into(&mut self.window);
        self.window.draw_box();
    }
}

impl Dialog for CombatResultWindow {
    fn window(&self) -> &Widget {
        &self.window
    }
    fn window_mut(&mut self) -> &mut Widget {
        &mut self.window
    }
}

