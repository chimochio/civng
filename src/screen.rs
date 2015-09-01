/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

//! Represent hex cells *in the context of a terminal UI*.

use std::collections::{HashSet, HashMap};
use std::cmp::{min, max};

// Re-export for doctests
pub use rustty::{Terminal, CellAccessor, HasPosition, HasSize, Cell, Style, Attr, Color};
use rustty::Pos as ScreenPos;
use rustty::ui::{Painter, Alignable, HorizontalAlign, VerticalAlign, Widget};

use hexpos::{Pos, Direction, OffsetPos};
use terrain::{Terrain, TerrainMap};
use map::LiveMap;
use unit::{Unit, Player};
use details_window::DetailsWindow;
use selection::Selection;

const CELL_WIDTH: usize = 7;
const CELL_HEIGHT: usize = 4;
// See diagram in HexCell's comment to understand why we have this offset.
const CELL_OFFSET_X: usize = 1;
const CELL_OFFSET_Y: usize = 0;

/// Returns the position of `pos` on the screen
///
/// The origin of the screen is assumed to be Pos::origin(). If its not, the `pos` you send has
/// to be translated first.
///
/// The screen pos given is the approximate center of the cell, as defined by `CELL_CENTER_COL`
/// and `CELL_CENTER_ROW`.
fn get_screenpos(pos: Pos) -> ScreenPos {
    let (mut spx, mut spy) = (CELL_OFFSET_X, CELL_OFFSET_Y);
    spx = ((spx as i32) + pos.x * (CELL_WIDTH as i32)) as usize;
    spy = ((spy as i32) - pos.y * ((CELL_HEIGHT / 2) as i32)) as usize;
    spy = ((spy as i32) + pos.z * ((CELL_HEIGHT / 2) as i32)) as usize;
    (spx, spy)
}

/*
 X     ╲
╱       ╲
╲       ╱
 ╲_____╱
 * Origin of the widget is at X.
 * Don't put contents in corners, you'll conflict with grid lines.
 */
struct HexCell {
    widget: Widget,
}

impl HexCell {
    pub fn new() -> HexCell {
        let widget = Widget::new(CELL_WIDTH, CELL_HEIGHT);
        HexCell {
            widget: widget,
        }
    }

    pub fn clear(&mut self) {
        self.widget.clear(Cell::default());
    }

    pub fn draw_into(&self, cells: &mut CellAccessor) {
        self.widget.draw_into(cells);
    }

    pub fn move_(&mut self, pos: Pos) {
        let (x, y) = get_screenpos(pos);
        self.widget.set_origin((x, y));
    }

    /// Highlight the cell with specified `color`.
    ///
    /// We proceed by setting the background color of peripherical cells. We don't highlight the
    /// whole background because it makes the contents of the cell very hard to read. We also don't
    /// highlight the cell's grid because of a technical problem: The upper line of the cell is
    /// not actually made of characters, it's made of the `Underline` attribute of the above cell.
    /// If we changed the color of that line, we would also change the color of the above cell's
    /// lower characters.
    pub fn highlight(&mut self, color: Color) {
        let (cols, rows) = self.widget.size();
        let mut doit = |x, y| {
            let cell = self.widget.get_mut(x, y).unwrap();
            cell.set_bg(Style::with_color(color));
        };
        for ix in 1..cols-1 {
            doit(ix, 0);
            doit(ix, rows-1);
        }
        for iy in 1..rows-1 {
            doit(0, iy);
            doit(cols-1, iy);
        }
    }

    pub fn draw_terrain(&mut self, terrain: Terrain) {
        let ch = terrain.map_char();
        let s: String = (0..5).map(|_| ch).collect();
        self.widget.printline(1, 0, &s);
        let cell = Cell::with_styles(Style::with_attr(Attr::Underline), Style::default());
        self.widget.printline_with_cell(1, 3, &s, cell);
    }

    pub fn draw_posmarker(&mut self, pos: OffsetPos) {
        self.widget.printline(1, 1, &pos.fmt());
    }

    pub fn draw_unit(&mut self, unit: &Unit, is_active: bool) {
        let mut cell = self.widget.get_mut(3, 2).unwrap();
        cell.set_ch(unit.map_symbol());
        let style = if unit.owner() != Player::Me {
                Style::with_color(Color::Red)
            }
            else if is_active {
                Style::with_color(Color::Blue)
            }
            else {
                Style::default()
            };
        cell.set_fg(style);
    }
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

    fn visible_cells(&self) -> VisiblePosIterator {
        VisiblePosIterator::new(self.topleft, self.term.cols(), self.term.rows())
    }

    /// Fills the screen with a hex grid.
    fn drawgrid(&mut self) {
        //  ╱     ╲
        // ╱       ╲_____
        // ╲       ╱
        //  ╲_____╱
        const SPAN_X: usize = 14;
        const SPAN_Y: usize = 4;
        let chars = [
            ('╱', 1, 0),
            ('╲', 7, 0),
            ('╱', 0, 1),
            ('╲', 8, 1),
            ('╲', 0, 2),
            ('╱', 8, 2),
            ('╲', 1, 3),
            ('╱', 7, 3),
        ];
        // +1 because we want the rightmost part of the grid to be there even if incomplete
        let rowrepeatcount = (self.term.rows() / SPAN_Y) + 1;
        let colrepeatcount = (self.term.cols() / SPAN_X) + 1;
        for rowrepeat in 0..rowrepeatcount {
            let basey = rowrepeat * SPAN_Y;
            for colrepeat in 0..colrepeatcount {
                let basex = colrepeat * SPAN_X;
                for &(ch, offset_x, offset_y) in chars.iter() {
                    if let Some(cell) = self.term.get_mut(basex + offset_x, basey + offset_y) {
                        cell.set_ch(ch);
                    }
                }
            }
        }
        // In addition to the vertical wavy lines, we also need to draw an intermittent horizontal
        // line on the top of the screen because the upper hex cells that only draw their bottom
        // parts are not drawn at all (and it's the "contents" part of the cell that is responsible
        // to draw the horizontal line, not drawgrid()). We compensate here.
        for colrepeat in 0..colrepeatcount {
            // +9 because we start at the edge of the hex cell we've just drawn
            let basex = colrepeat * SPAN_X + 9;
            for i in 0..5 {
                if let Some(cell) = self.term.get_mut(basex + i, 1) {
                    cell.set_fg(Style::with_attr(Attr::Underline));
                }
            }
        }
    }

    /// Draws everything we're supposed to draw.
    ///
    /// `map` is the terrain map we want to draw and `unitpos` is the position of the test unit
    /// we're moving around.
    pub fn draw(
            &mut self,
            map: &LiveMap,
            selection: &Selection,
            popup: Option<&mut Widget>) {
        let yellowpos = match selection.unit_id {
            Some(uid) => {
                let active_unit = map.units().get(uid);
                active_unit.reachable_pos(map.terrain(), map.units())
            }
            None => HashMap::new(),
        };
        let mut cell = HexCell::new();
        for pos in self.visible_cells() {
            cell.clear();
            let relpos = self.relpos(pos);
            if self.has_option(DisplayOption::PosMarkers) {
                cell.draw_posmarker(pos.to_offset_pos());
            }
            let terrain = map.terrain().get_terrain(pos);
            cell.draw_terrain(terrain);
            if let Some(unit_id) = map.units().unit_at_pos(pos) {
                let unit = map.units().get(unit_id);
                let is_active = selection.is_unit_active(unit.id());
                cell.draw_unit(unit, is_active);
            }
            if selection.pos.is_some() && pos == selection.pos.unwrap() {
                cell.highlight(Color::Blue)
            }
            else if yellowpos.contains_key(&pos) {
                cell.highlight(Color::Yellow);
            }
            cell.move_(relpos);
            cell.draw_into(&mut self.term);
        }
        self.drawgrid();
        self.details_window.draw_into(&mut self.term);
        if let Some(w) = popup {
            w.align(&self.term, HorizontalAlign::Middle, VerticalAlign::Middle, 0);
            w.draw_into(&mut self.term);
        }
        let _ = self.term.swap_buffers();
    }
}

