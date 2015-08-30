/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

//! Represent hex cells *in the context of a terminal UI*.

use std::collections::HashSet;
use std::cmp::{min, max};

use num::integer::Integer;

// Re-export for doctests
pub use rustty::{Terminal, CellAccessor, HasSize, Cell, Style, Attr, Color};
use rustty::Pos as ScreenPos;
use rustty::ui::{Painter, Alignable, HorizontalAlign, VerticalAlign, Widget};

use hexpos::{Pos, Direction, OffsetPos};
use terrain::{TerrainMap};
use map::LiveMap;
use unit::{Units, Player};
use details_window::DetailsWindow;

const CELL_WIDTH: usize = 7;
const CELL_HEIGHT: usize = 4;
const CELL_CENTER_COL: usize = 4;
const CELL_CENTER_ROW: usize = 1;

/// Returns the position of `pos` on the screen
///
/// The origin of the screen is assumed to be Pos::origin(). If its not, the `pos` you send has
/// to be translated first.
///
/// The screen pos given is the approximate center of the cell, as defined by `CELL_CENTER_COL`
/// and `CELL_CENTER_ROW`.
fn get_screenpos(pos: Pos) -> ScreenPos {
    let (mut spx, mut spy) = (CELL_CENTER_COL, CELL_CENTER_ROW);
    spx = ((spx as i32) + pos.x * (CELL_WIDTH as i32)) as usize;
    spy = ((spy as i32) - pos.y * ((CELL_HEIGHT / 2) as i32)) as usize;
    spy = ((spy as i32) + pos.z * ((CELL_HEIGHT / 2) as i32)) as usize;
    (spx, spy)
}

pub fn get_contents_screenpos(pos: Pos, dx: i8, dy: i8) -> ScreenPos {
    let (mut spx, mut spy) = get_screenpos(pos);
    spx = ((spx as isize) + (dx as isize)) as usize;
    spy = ((spy as isize) + (dy as isize)) as usize;
    (spx, spy)
}

struct VisiblePosIterator {
    screen_cols: usize,
    screen_rows: usize,
    origin: Pos,
    leftmost: Pos,
    current: Pos,
    direction: Direction,
}

impl VisiblePosIterator {
    fn new(topleft: Pos, screen_cols: usize, screen_rows: usize) -> VisiblePosIterator {
        VisiblePosIterator{
            screen_cols: screen_cols,
            screen_rows: screen_rows,
            origin: topleft,
            leftmost: topleft,
            current: topleft,
            direction: Direction::SouthEast,
        }
    }
}

impl Iterator for VisiblePosIterator {
    type Item = Pos;

    fn next(&mut self) -> Option<Pos> {
        let rpos = self.current.translate(self.origin.neg());
        let (spx, spy) = get_screenpos(rpos);
        if spy < self.screen_rows && spx < self.screen_cols {
            let result = self.current;
            self.current = self.current.neighbor(self.direction);
            self.direction = if self.direction == Direction::SouthEast { Direction::NorthEast } else { Direction:: SouthEast };
            Some(result)
        }
        else {
            self.leftmost = self.leftmost.neighbor(Direction::South);
            let rpos = self.leftmost.translate(self.origin.neg());
            let (spx, spy) = get_screenpos(rpos);
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
    topleft: Pos,
    pub details_window: DetailsWindow,
}

impl Screen {
    pub fn new() -> Screen {
        let term = Terminal::new().unwrap();
        let details_window = DetailsWindow::new(&term);
        Screen {
            term: term,
            options: HashSet::new(),
            topleft: Pos::origin(),
            details_window: details_window,
        }
    }

    /// Size of the terminal in number of hex cells that fits in it.
    pub fn size_in_cells(&self) -> (usize, usize) {
        let (cols, rows) = self.term.size();
        (cols / CELL_WIDTH, rows / CELL_HEIGHT)
    }

    fn relpos(&self, pos: Pos) -> Pos {
        pos.translate(self.topleft.neg())
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

    /// Scrolls the visible part of the map by `by`.
    ///
    /// # Examples
    ///
    /// ```
    /// use civng::screen::Screen;
    /// use civng::hexpos::{Pos, Direction};
    ///
    /// let mut screen = Screen::new();
    /// // Scrolls the screen SW by 3 cells.
    /// screen.scroll(Pos::vector(Direction::SouthEast).amplify(3));
    /// ```
    pub fn scroll(&mut self, by: Pos) {
        self.topleft = self.topleft.translate(by);
    }

    /// Scrolls visible part of the map so that `pos` is at the center of the screen.
    ///
    /// We *don't* go past `map`'s borders, so if we try to call this method with a `pos` that is
    /// near an edge of `map`, our screen will not be exactly centered on `pos`, but we can
    /// guarantee that it will be visible.
    ///
    /// # Examples
    ///
    /// ```
    /// use civng::screen::Screen;
    /// use civng::terrain::TerrainMap;
    /// use civng::hexpos::{OffsetPos};
    ///
    /// let mut screen = Screen::new();
    /// let map = TerrainMap::empty_map(42, 42);
    /// let pos = OffsetPos::new(21, 21).to_pos();
    /// // Our screen now shows the center of the terrain map
    /// screen.center_on_pos(pos, &map);
    /// ```
    pub fn center_on_pos(&mut self, pos: Pos, map: &TerrainMap) {
        let (width, height) = self.size_in_cells();
        let (map_width, map_height) = map.size();
        let max_x = map_width - width as i32;
        let max_y = map_height - height as i32;
        let target_dx = (width / 2) as i32;
        let target_dy = (height / 2) as i32;
        let opos = pos.to_offset_pos();
        let target_x = max(min(opos.x - target_dx, max_x), 0);
        let target_y = max(min(opos.y - target_dy, max_y), 0);
        self.topleft = OffsetPos::new(target_x, target_y).to_pos();
    }

    // ">= 0" checks are useless because of usize, but it seems dangerous to leave them out. If we
    // ever adopt a signed int, we might introduce a bug here without knowing.
    #[allow(unused_comparisons)]
    fn is_pos_visible(&self, pos: Pos) -> bool {
        let (x, y) = get_screenpos(self.relpos(pos));
        y >= 0 && x >= 0 && y < self.term.rows() && x < self.term.cols()
    }

    fn visible_cells(&self) -> VisiblePosIterator {
        VisiblePosIterator::new(self.topleft, self.term.cols(), self.term.rows())
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
        // +1 because we want the rightmost part of the grid to be there even if incomplete
        let colrepeatcount = (self.term.cols() / linewidth) + 1;
        for y in 0..self.term.rows() {
            for colrepeat in 0..colrepeatcount {
                let x = colrepeat * linewidth;
                let (_, lineno) = y.div_rem(&lines.len());
                let line = lines[lineno];
                self.term.printline(x, y, line);
            }
        }
    }

    /// Draws position marks in each hex cell on the screen.
    fn drawposmarkers(&mut self) {
        for pos in self.visible_cells() {
            let (x, y) = get_contents_screenpos(self.relpos(pos), -3, 0);
            self.term.printline(x, y, &pos.to_offset_pos().fmt());
        }
    }

    /// Draws terrain information in each visible cell.
    fn drawterrain(&mut self, map: &TerrainMap) {
        for pos in self.visible_cells() {
            let ch = map.get_terrain(pos).map_char();
            let s: String = (0..5).map(|_| ch).collect();
            let (x, y) = get_contents_screenpos(self.relpos(pos), -2, -1);
            self.term.printline(x, y, &s);
            let cell = Cell::with_styles(Style::with_attr(Attr::Underline), Style::default());
            let (x, y) = get_contents_screenpos(self.relpos(pos), -2, 2);
            self.term.printline_with_cell(x, y, &s, cell);
        }
    }

    /// Draws a 'X' at specified `pos`.
    fn drawunits(&mut self, units: &Units, active_unit_index: Option<usize>) {
        for unit in units.all_units() {
            let pos = unit.pos();
            if self.is_pos_visible(pos) {
                let relpos = self.relpos(pos);
                let (x, y) = get_contents_screenpos(relpos, 0, 1);
                match self.term.get_mut(x, y) {
                    Some(cell) => {
                        cell.set_ch(unit.map_symbol());
                        let style = if unit.owner() != Player::Me {
                                Style::with_color(Color::Red)
                            }
                            else if active_unit_index.is_some() && unit.id() == active_unit_index.unwrap() {
                                Style::with_color(Color::Blue)
                            }
                            else {
                                Style::default()
                            };
                        cell.set_fg(style);
                    },
                    None => {}, // ignore
                };
            };
        }
    }

    /// Draws everything we're supposed to draw.
    ///
    /// `map` is the terrain map we want to draw and `unitpos` is the position of the test unit
    /// we're moving around.
    pub fn draw(&mut self, map: &LiveMap, active_unit_index: Option<usize>, popup: Option<&mut Widget>) {
        self.drawgrid();
        if self.has_option(DisplayOption::PosMarkers) {
            self.drawposmarkers();
        }
        self.drawterrain(map.terrain());
        self.drawunits(map.units(), active_unit_index);
        self.details_window.draw_into(&mut self.term);
        match popup {
            Some(w) => {
                w.align(&self.term, HorizontalAlign::Middle, VerticalAlign::Middle, 0);
                w.draw_into(&mut self.term);
            }
            None => (),
        }
        let _ = self.term.swap_buffers();
    }
}

