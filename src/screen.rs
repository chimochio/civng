/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

use num::integer::Integer;
use rustty::{Terminal};
use hexpos::{Pos, Direction};
use map::{TerrainMap};

const CELL_WIDTH: usize = 7;
const CELL_HEIGHT: usize = 4;
const CELL_CENTER_COL: usize = 4;
const CELL_CENTER_ROW: usize = 1;

#[derive(Copy, Clone)]
pub struct ScreenPos {
    row: usize,
    col: usize,
}

impl ScreenPos {
    pub fn new(row: usize, col: usize) -> ScreenPos {
        ScreenPos {
            row: row,
            col: col,
        }
    }

    pub fn astuple(&self) -> (usize, usize) {
        (self.row, self.col)
    }
}

/// Representation of a Cell on screen
#[derive(Copy, Clone)]
pub struct ScreenCell {
    pos: Pos,
    screenpos: ScreenPos,
}

impl ScreenCell {
    pub fn refcell () -> ScreenCell {
        ScreenCell{
            pos: Pos::new(0, 0, 0),
            screenpos: ScreenPos::new(CELL_CENTER_ROW, CELL_CENTER_COL),
        }
    }

    pub fn neighbor(&self, direction: Direction) -> ScreenCell {
        let p = Pos::new(0, 0, 0).neighbor(direction);
        self.relative_cell(p)
    }

    pub fn relative_cell(&self, relative_pos: Pos) -> ScreenCell {
        let mut p = self.pos;
        p.x += relative_pos.x;
        p.y += relative_pos.y;
        p.z += relative_pos.z;
        let mut sp = self.screenpos;
        sp.col = ((sp.col as i32) + relative_pos.x * (CELL_WIDTH as i32)) as usize;
        sp.row = ((sp.row as i32) - relative_pos.y * ((CELL_HEIGHT / 2) as i32)) as usize;
        sp.row = ((sp.row as i32) + relative_pos.z * ((CELL_HEIGHT / 2) as i32)) as usize;
        ScreenCell { pos: p, screenpos: sp }
    }

    pub fn contents_screenpos(&self, dy: i8, dx: i8) -> ScreenPos{
        let mut sp = self.screenpos;
        sp.row = ((sp.row as isize) + (dy as isize)) as usize;
        sp.col = ((sp.col as isize) + (dx as isize)) as usize;
        sp
    }
}

struct VisibleCellIterator {
    screen_cols: usize,
    screen_rows: usize,
    leftmost: ScreenCell,
    current: ScreenCell,
    direction: Direction,
}

impl VisibleCellIterator {
    fn new(topleft: ScreenCell, screen_cols: usize, screen_rows: usize) -> VisibleCellIterator {
        VisibleCellIterator{
            screen_cols: screen_cols,
            screen_rows: screen_rows,
            leftmost: topleft,
            current: topleft,
            direction: Direction::SouthEast,
        }
    }
}

impl Iterator for VisibleCellIterator {
    type Item = ScreenCell;

    fn next(&mut self) -> Option<ScreenCell> {
        let screenpos = self.current.screenpos;
        if screenpos.row < self.screen_rows && screenpos.col < self.screen_cols {
            let result = self.current;
            self.current = self.current.neighbor(self.direction);
            self.direction = if self.direction == Direction::SouthEast { Direction::NorthEast } else { Direction:: SouthEast };
            Some(result)
        }
        else {
            self.leftmost = self.leftmost.neighbor(Direction::South);
            let screenpos = self.leftmost.screenpos;
            if screenpos.row < self.screen_rows && screenpos.col < self.screen_cols {
                self.current = self.leftmost;
                self.direction = Direction::SouthEast;
                Some(self.current)
            }
            else {
                None
            }
        }
    }
}

pub struct Screen {
    pub term: Terminal,
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            term: Terminal::new().unwrap(),
        }
    }

    pub fn printline(&mut self, screenpos: ScreenPos, line: &str) {
        for (index, ch) in line.chars().enumerate() {
            let x = screenpos.col + index;
            if x >= self.term.cols() {
                break;
            }
            self.term[(screenpos.row, x)].set_ch(ch);
        }
    }

    pub fn drawgrid(&mut self) {
        let lines: [&str; 4] = [
            " /     \\      ",
            "/       \\ _ _ ",
            "\\       /     ",
            " \\ _ _ /      ",
        ];
        let (cols, rows) = self.term.size();
        let colrepeatcount = cols / lines[0].len();
        for y in 0..rows-1 {
            for colrepeat in 0..colrepeatcount {
                let x = colrepeat * lines[0].len();
                let (_, lineno) = y.div_rem(&lines.len());
                let line = lines[lineno];
                self.printline(ScreenPos{ row: y, col: x }, line);
            }
        }
    }

    pub fn drawposmarkers(&mut self) {
        let cellit = VisibleCellIterator::new(ScreenCell::refcell(), self.term.cols(), self.term.rows());
        for sc in cellit {
            self.printline(sc.contents_screenpos(0, -3), &sc.pos.to_offset_pos().fmt());
        }
    }

    pub fn drawwalls(&mut self, map: &TerrainMap) {
        let cellit = VisibleCellIterator::new(ScreenCell::refcell(), self.term.cols(), self.term.rows());
        for sc in cellit {
            let ch = map.get_terrain(sc.pos).map_char();
            let s: String = (0..3).map(|_| ch).collect();
            self.printline(sc.contents_screenpos(-1, -1), &s);
        }
    }

    pub fn drawunit(&mut self, pos: Pos) {
        let refcell = ScreenCell::refcell();
        let sc = refcell.relative_cell(pos);
        self.term[sc.contents_screenpos(1, 0).astuple()].set_ch('X');
    }

    pub fn draw(&mut self, map: &TerrainMap, unitpos: Pos) {
        self.drawgrid();
        self.drawposmarkers();
        self.drawwalls(map);
        self.drawunit(unitpos);
        let _ = self.term.swap_buffers();
    }
}

