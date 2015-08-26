/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use rustty::{CellAccessor, Cell, HasSize};
use rustty::ui::{Painter, Widget, Alignable, HorizontalAlign, VerticalAlign};

use map::LiveMap;

pub struct DetailsWindow {
    window: Widget,
}

impl DetailsWindow {
    pub fn new(parent: &HasSize) -> DetailsWindow {
        let mut window = Widget::new(16, 7);
        window.align(parent, HorizontalAlign::Right, VerticalAlign::Bottom, 0);
        DetailsWindow {
            window: window,
        }
    }

    pub fn draw_into(&self, cells: &mut CellAccessor) {
        self.window.draw_into(cells);
    }

    pub fn update(&mut self, active_unit_id: Option<usize>, map: &LiveMap, turn: u16, scrollmode: bool) {
        let turn_line = format!("Turn {}", turn);
        let sm_line = (if scrollmode { "Scroll Mode" } else { "" }).to_owned();
        let lines = match active_unit_id {
            Some(uid) => {
                let unit = map.units().get(uid);
                let terrain = map.terrain().get_terrain(unit.pos());
                [
                    unit.name().to_owned(),
                    format!("MV {} / HP {}", unit.movements(), unit.hp()),
                    terrain.name().to_owned(),
                    turn_line,
                    sm_line,
                ]
            }
            None => [
                "".to_owned(),
                "".to_owned(),
                "".to_owned(),
                turn_line,
                sm_line,
            ],
        };
        self.window.clear(Cell::default());
        for (index, line) in lines.iter().enumerate() {
            self.window.printline(2, index+1, line);
        }
        self.window.draw_box();
    }
}
