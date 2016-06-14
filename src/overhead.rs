// Copyright 2016 Virgil Dupras
//
// This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
// which should be included with this package. The terms are also available at
// http://www.gnu.org/licenses/gpl-3.0.html
//

use rustty::{CellAccessor, Color};

use terrain::TerrainMap;
use hexpos::{OffsetPos, Pos};

pub fn draw_overhead_map(target: &mut CellAccessor, map: &TerrainMap, selected_pos: Option<Pos>) {
    let (mapw, maph) = map.size();
    for ih in 0..maph {
        for iw in 0..mapw {
            let pos = OffsetPos::new(iw, ih).to_pos();
            let terrain = map.get_terrain(pos);
            if let Some(cell) = target.get_mut(iw as usize, ih as usize) {
                cell.set_ch(terrain.map_char());
                if selected_pos == Some(pos) {
                    cell.set_bg(Color::Blue);
                }
            }
        }
    }
}
