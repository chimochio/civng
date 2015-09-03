/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use rustty::{CellAccessor, Cell, HasSize};
use rustty::ui::{Painter, Widget, Alignable, HorizontalAlign, VerticalAlign};

use hexpos::Pos;
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

    pub fn update(&mut self, selected_pos: Option<Pos>, map: &LiveMap, turn: u16, movemode: &str) {
        let turn_line = format!("Turn {}", turn);
        let (terrain_name, maybe_unit_id) = match selected_pos {
            Some(pos) => (map.terrain().get_terrain(pos).name().to_owned(), map.units().unit_at_pos(pos)),
            None => ("".to_owned(), None)
        };
        let (unit_name, unit_stats) = if let Some(uid) = maybe_unit_id {
            let unit = map.units().get(uid);
            (unit.name(), format!("MV {} / HP {}", unit.movements(), unit.hp()))
        }
        else {
            ("", "".to_owned())
        };
        let lines = [
            unit_name,
            &unit_stats[..],
            &terrain_name[..],
            &turn_line[..],
            movemode,
        ];
        self.window.clear(Cell::default());
        for (index, line) in lines.iter().enumerate() {
            self.window.printline(2, index+1, line);
        }
        self.window.draw_box();
    }
}
