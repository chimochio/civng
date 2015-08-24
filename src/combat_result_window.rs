/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use rustty::{CellAccessor, Cell};
use rustty::ui::{Painter, Window, HorizontalAlign, VerticalAlign};

use dialog::Dialog;
use combat::CombatResult;

pub struct CombatResultWindow {
    window: Window,
}

impl CombatResultWindow {
    pub fn new() -> CombatResultWindow {
        let window = Window::new(35, 10);
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
        let s = "Press any key to continue";
        let x = self.window.halign_line(s, HorizontalAlign::Middle, 1);
        let y = self.window.valign_line(s, VerticalAlign::Bottom, 1);
        self.window.printline(x, y, s);
        self.window.draw_box();
    }
}

impl Dialog for CombatResultWindow {
    fn window(&self) -> &Window {
        &self.window
    }
    fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
}

