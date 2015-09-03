/* Copyright 2015 Virgil Dupras
 *
 * This software is licensed under the "GPLv3" License as described in the "LICENSE" file,
 * which should be included with this package. The terms are also available at
 * http://www.gnu.org/licenses/gpl-3.0.html
 */

//! Represent hex cells *in the context of a terminal UI*.

use std::collections::{HashSet, HashMap};
use std::cmp::{min, max};

use num::integer::Integer;

// Re-export for doctests
pub use rustty::{Terminal, CellAccessor, HasPosition, HasSize, Cell, Style, Attr, Color};
use rustty::Pos as ScreenPos;
use rustty::ui::{Painter, Alignable, HorizontalAlign, VerticalAlign, Widget};

use hexpos::{Pos, OffsetPos};
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

/// Size of the terminal in number of hex cells that fits in it.
fn size_in_cells(term: &Terminal) -> (usize, usize) {
    let (cols, rows) = term.size();
    // cols -2 because of the overhead of the wavy lines. Without this overhead counting, we
    // get incomplete borders.
    // rows -2 also because of wavy cell placement overhead
    ((cols - 2)/ CELL_WIDTH, (rows - 2) / CELL_HEIGHT)
}

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
    pos: Pos,
    widget: Widget,
}

impl HexCell {
    pub fn new(pos: Pos) -> HexCell {
        let mut widget = Widget::new(CELL_WIDTH, CELL_HEIGHT);
        let (x, y) = get_screenpos(pos);
        widget.set_origin((x, y));
        HexCell {
            pos: pos,
            widget: widget,
        }
    }

    pub fn pos(&self) -> Pos {
        self.pos
    }

    pub fn clear(&mut self) {
        self.widget.clear(Cell::default());
    }

    pub fn draw_into(&self, cells: &mut CellAccessor) {
        self.widget.draw_into(cells);
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
    cells: Vec<HexCell>,
    options: HashSet<DisplayOption>,
    /// Cell at the top-left corner of the screen
    topleft: Pos,
    /// Size of the map during the last draw call.
    map_size: (i32, i32),
    pub details_window: DetailsWindow,
}

impl Screen {
    pub fn new() -> Screen {
        let term = Terminal::new().unwrap();
        let details_window = DetailsWindow::new(&term);
        let (screenw, screenh) = size_in_cells(&term);
        let mut cells = Vec::new();
        for iy in 0..screenh {
            for ix in 0..screenw {
                let pos = OffsetPos::new(ix as i32, iy as i32).to_pos();
                cells.push(HexCell::new(pos));
            }
        }
        Screen {
            term: term,
            cells: cells,
            options: HashSet::new(),
            topleft: Pos::origin(),
            map_size: (0, 0),
            details_window: details_window,
        }
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

    pub fn scroll_to(&mut self, topleft: Pos) {
        let mut opos = topleft.to_offset_pos();
        let (screenw, screenh) = size_in_cells(&self.term);
        let (mapw, maph) = self.map_size;
        opos.y = min(opos.y, maph - screenh as i32);
        opos.x = min(opos.x, mapw - screenw as i32);
        opos.y = max(opos.y, 0);
        opos.x = max(opos.x, 0);
        self.topleft = opos.to_pos();
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
        let target = self.topleft.translate(by);
        self.scroll_to(target);
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
        let (width, height) = size_in_cells(&self.term);
        let (map_width, map_height) = map.size();
        let max_x = map_width - width as i32;
        let max_y = map_height - height as i32;
        let target_dx = (width / 2) as i32;
        let target_dy = (height / 2) as i32;
        let opos = pos.to_offset_pos();
        let target_x = max(min(opos.x - target_dx, max_x), 0);
        let target_y = max(min(opos.y - target_dy, max_y), 0);
        self.scroll_to(OffsetPos::new(target_x, target_y).to_pos());
    }

    /// Fills the screen with a hex grid.
    fn drawgrid(&mut self) {
        //  ╱     ╲
        // ╱       ╲
        // ╲       ╱
        //  ╲_____╱
        let otopleft = self.topleft.to_offset_pos();
        let is_oddx = otopleft.x.div_rem(&2).1 == 1;
        let (mapw, maph) = self.map_size;
        let chars = [
            ('╱', 1),
            ('╱', 0),
            ('╲', 0),
            ('╲', 1),
        ];
        let (screenx, screeny) = size_in_cells(&self.term);
        let is_at_top = otopleft.y == 0 && !is_oddx;
        let is_at_bottom = otopleft.y + screeny as i32 >= maph;
        let is_at_left = otopleft.x == 0;
        let is_at_right = otopleft.x + screenx as i32 >= mapw;
        // +1 because we want to close the last cell by drawing its right border, not only its
        // left one.
        for colrepeat in 0..screenx+1 {
            let basex = colrepeat * CELL_WIDTH;
            let skipcount = if colrepeat.div_rem(&2).1 == 1 { 2 } else { 0 };
            let mut takecount = screeny * CELL_HEIGHT + 2;
            if colrepeat == 0 || (is_at_bottom && is_oddx) {
                // The colrepeat==0  gives us a "rounded" corner.
                // The bottom check ensures that we don't draw out of bounds cells, which can
                // happen if we scroll to the bottom and have an odd topleft.
                takecount -= 2;
            }
            let char_iter = chars.iter().cycle().skip(skipcount).enumerate();
            for (y, &(ch, offset_x)) in char_iter.take(takecount) {
                if let Some(cell) = self.term.get_mut(basex + offset_x, y) {
                    let top_limit = is_at_top && y < 2;
                    let bottom_limit = colrepeat > 0 && is_at_bottom && y >= takecount - 2;
                    let left_limit = is_at_left && colrepeat == 0;
                    let right_limit = is_at_right && colrepeat == screenx;
                    if top_limit || bottom_limit || left_limit || right_limit {
                        cell.set_fg(Style::with_color(Color::Red));
                    }
                    cell.set_ch(ch);
                }
            }
        }
        // In addition to the vertical wavy lines, we also need to draw an intermittent horizontal
        // line on the top of the screen because the upper hex cells that only draw their bottom
        // parts are not drawn at all (and it's the "contents" part of the cell that is responsible
        // to draw the horizontal line, not drawgrid()). We compensate here.
        for colrepeat in 0..screenx {
            if colrepeat.div_rem(&2).1 == 1 {
                let basex = colrepeat * CELL_WIDTH + 2;
                for i in 0..5 {
                    if let Some(cell) = self.term.get_mut(basex + i, 1) {
                        cell.set_fg(Style::with_attr(Attr::Underline));
                    }
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
        let _ = self.term.clear();
        self.map_size = map.terrain().size();
        let reachablepos = match selection.unit_id {
            Some(uid) => {
                map.reachable_pos(uid)
            }
            None => HashMap::new(),
        };
        let with_posmarkers = self.has_option(DisplayOption::PosMarkers);
        for cell in self.cells.iter_mut() {
            let pos = cell.pos().translate(self.topleft);
            cell.clear();
            let terrain = map.terrain().get_terrain(pos);
            // Can happen if out top left has a odd x and that we're at the bottom of the map.
            if terrain == Terrain::OutOfBounds {
                continue;
            }
            if with_posmarkers {
                cell.draw_posmarker(pos.to_offset_pos());
            }
            cell.draw_terrain(terrain);
            if let Some(unit_id) = map.units().unit_at_pos(pos) {
                let unit = map.units().get(unit_id);
                let is_active = selection.is_unit_active(unit.id());
                cell.draw_unit(unit, is_active);
            }
            if selection.pos.is_some() && pos == selection.pos.unwrap() {
                cell.highlight(Color::Blue)
            }
            else if reachablepos.contains_key(&pos) {
                let mut color = Color::Yellow;
                if let Some(u) = map.units().get_at_pos(pos) {
                    if u.owner() != Player::Me {
                        color = Color::Red;
                    }
                }
                cell.highlight(color);
            }
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

