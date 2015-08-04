/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

//! Represent hex cells *in the context of a terminal UI*.

use std::collections::HashSet;

use num::integer::Integer;

// Re-export for doctests
pub use rustty::{Terminal, CellAccessor};
use rustty::Pos as ScreenPos;
use rustty::ui::{Window, Painter};

use hexpos::{Pos, Direction};
use map::{TerrainMap};

const CELL_WIDTH: usize = 7;
const CELL_HEIGHT: usize = 4;
const CELL_CENTER_COL: usize = 4;
const CELL_CENTER_ROW: usize = 1;

/// Representation of a Cell on screen
///
/// That is, the mapping of a hex `Pos` to a `ScreenPos`. Because hex `Pos` are pure, wwe have
/// to anchor them somehow to the screen. We do so by placing our origin somewhere on a
/// `ScreenPos`. Then, it's only a matter of deducing other neighboring `ScreenCell`s.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ScreenCell {
    /// "Pure" hex position
    pos: Pos,
    /// Where `pos` is anchored to the screen
    screenpos: ScreenPos,
}

impl ScreenCell {
    /// Reference (origin) cell.
    ///
    /// That is, the origin `Pos`, mapped to the top-left corner of the screen.
    pub fn refcell(origin: Pos) -> ScreenCell {
        ScreenCell{
            pos: origin,
            screenpos: (CELL_CENTER_ROW, CELL_CENTER_COL),
        }
    }

    /// Returns `self`'s neighboring cell in the given `direction`.
    pub fn neighbor(&self, direction: Direction) -> ScreenCell {
        let p = Pos::new(0, 0, 0).neighbor(direction);
        self.relative_cell(p)
    }

    /// Returns a cell relative to `self` by `relative_pos`
    ///
    /// # Examples
    /// ```
    /// use civng::hexpos::{Direction, Pos};
    /// use civng::screen::ScreenCell;
    ///
    /// let refcell = ScreenCell::refcell(Pos::origin());
    /// let mut cell1 = refcell;
    /// for _ in 0..3 {
    ///     cell1 = cell1.neighbor(Direction::North);
    /// }
    /// let cell2 = refcell.relative_cell(Pos::origin().neighbor(Direction::North).amplify(3));
    /// assert_eq!(cell1, cell2);
    /// ```
    pub fn relative_cell(&self, relative_pos: Pos) -> ScreenCell {
        let mut p = self.pos;
        p.x += relative_pos.x;
        p.y += relative_pos.y;
        p.z += relative_pos.z;
        let (mut spx, mut spy) = self.screenpos;
        spx = ((spx as i32) + relative_pos.x * (CELL_WIDTH as i32)) as usize;
        spy = ((spy as i32) - relative_pos.y * ((CELL_HEIGHT / 2) as i32)) as usize;
        spy = ((spy as i32) + relative_pos.z * ((CELL_HEIGHT / 2) as i32)) as usize;
        ScreenCell { pos: p, screenpos: (spx, spy) }
    }

    /// Returns a `ScreenPos` relative to `self`.
    ///
    /// This allows us to easily change the contents of the cell.
    ///
    /// # Examples
    /// ```
    /// use civng::hexpos::Pos;
    /// use civng::screen::{Terminal, ScreenCell};
    ///
    /// let mut term = Terminal::new().unwrap();
    /// let cell = ScreenCell::refcell(Pos::origin());
    /// let pos = cell.contents_screenpos(1, 3);
    /// // Prints a 'X' in the upper-center of the tile.
    /// term[pos.astuple()].set_ch('X');
    /// ```
    pub fn contents_screenpos(&self, dy: i8, dx: i8) -> ScreenPos {
        let (mut spx, mut spy) = self.screenpos;
        spy = ((spy as isize) + (dy as isize)) as usize;
        spx = ((spx as isize) + (dx as isize)) as usize;
        (spx, spy)
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
        let (spx, spy) = self.current.screenpos;
        if spy < self.screen_rows && spx < self.screen_cols {
            let result = self.current;
            self.current = self.current.neighbor(self.direction);
            self.direction = if self.direction == Direction::SouthEast { Direction::NorthEast } else { Direction:: SouthEast };
            Some(result)
        }
        else {
            self.leftmost = self.leftmost.neighbor(Direction::South);
            let (spx, spy) = self.leftmost.screenpos;
            if spy < self.screen_rows && spx < self.screen_cols {
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

/// Various display options that can be enabled in `Screen`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayOption {
    /// Show positional markers in each hex cell.
    PosMarkers,
}
/// Takes care of drawing everything we need to draw on screen.
///
/// This wraps's rustty's `Terminal` singleton, so any dealing we have with the terminal has to
/// go through this struct.
pub struct Screen {
    pub term: Terminal,
    options: HashSet<DisplayOption>,
    /// Cell at the top-left corner of the screen
    refcell: ScreenCell,
    pub details_window: Window,
}

impl Screen {
    pub fn new() -> Screen {
        let term = Terminal::new().unwrap();
        let (cols, rows) = term.size();
        let details_window = Window::new(cols-15, rows-4, 15, 4);
        Screen {
            term: term,
            options: HashSet::new(),
            refcell: ScreenCell::refcell(Pos::origin()),
            details_window: details_window,
        }
    }

    pub fn printline(&mut self, screenpos: ScreenPos, line: &str) {
        let (x, y) = screenpos;
        self.term.printline(x, y, line);
    }

    pub fn has_option(&self, option:DisplayOption) -> bool {
        self.options.contains(&option)
    }

    pub fn toggle_option(&mut self, option: DisplayOption) {
        if self.options.contains(&option) {
            self.options.remove(&option);
        }
        else {
            self.options.insert(option);
        }
    }

    pub fn scroll(&mut self, by: Pos) {
        self.refcell = ScreenCell::refcell(self.refcell.pos.translate(by));
    }

    // ">= 0" checks are useless because of usize, but it seems dangerous to leave them out. If we
    // ever adopt a signed int, we might introduce a bug here without knowing.
    #[allow(unused_comparisons)]
    fn is_cell_visible(&self, cell: ScreenCell) -> bool {
        let (x, y) = cell.screenpos;
        y >= 0 && x >= 0 && y < self.term.rows() && x < self.term.cols()
    }

    fn visible_cells(&self) -> VisibleCellIterator {
        VisibleCellIterator::new(self.refcell, self.term.cols(), self.term.rows())
    }

    /// Fills the screen with a hex grid.
    fn drawgrid(&mut self) {
        let lines: [&str; 4] = [
            " ╱     ╲      ",
            "╱       ╲_____",
            "╲       ╱     ",
            " ╲_____╱      ",
        ];
        // Don't use len(), it counts *bytes*.
        let linewidth = lines[0].chars().count();
        let colrepeatcount = self.term.cols() / linewidth;
        for y in 0..self.term.rows() {
            for colrepeat in 0..colrepeatcount {
                let x = colrepeat * linewidth;
                let (_, lineno) = y.div_rem(&lines.len());
                let line = lines[lineno];
                self.printline((y, x), line);
            }
        }
    }

    /// Draws position marks in each hex cell on the screen.
    fn drawposmarkers(&mut self) {
        for sc in self.visible_cells() {
            self.printline(sc.contents_screenpos(0, -3), &sc.pos.to_offset_pos().fmt());
        }
    }

    /// Draws terrain information in each visible cell.
    fn drawwalls(&mut self, map: &TerrainMap) {
        for sc in self.visible_cells() {
            let ch = map.get_terrain(sc.pos).map_char();
            let s: String = (0..3).map(|_| ch).collect();
            self.printline(sc.contents_screenpos(-1, -1), &s);
        }
    }

    /// Draws a 'X' at specified `pos`.
    fn drawunit(&mut self, pos: Pos) {
        let sc = self.refcell.relative_cell(pos.translate(self.refcell.pos.neg()));
        if self.is_cell_visible(sc) {
            let (x, y) = sc.contents_screenpos(1, 0);
            match self.term.get_mut(x, y) {
                Some(cell) => { cell.set_ch('X'); },
                None => {}, // ignore
            };
        };
    }

    /// Draws everything we're supposed to draw.
    ///
    /// `map` is the terrain map we want to draw and `unitpos` is the position of the test unit
    /// we're moving around.
    pub fn draw(&mut self, map: &TerrainMap, unitpos: Pos) {
        self.drawgrid();
        if self.has_option(DisplayOption::PosMarkers) {
            self.drawposmarkers();
        }
        self.drawwalls(map);
        self.drawunit(unitpos);
        self.details_window.draw_into(&mut self.term);
        let _ = self.term.swap_buffers();
    }
}

