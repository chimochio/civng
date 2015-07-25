use num::integer::Integer;
use rustty::{Terminal, Event};

extern crate num;
extern crate rustty;

const CELL_WIDTH: usize = 7;
const CELL_HEIGHT: usize = 4;
const CELL_CENTER_COL: usize = 4;
const CELL_CENTER_ROW: usize = 1;

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    North,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
}

#[derive(Copy, Clone)]
struct Pos {
    x: i32,
    y: i32,
    z: i32,
}

impl Pos {
    fn new(x: i32, y: i32, z: i32) -> Pos {
        Pos {
            x: x,
            y: y,
            z: z,
        }
    }

    fn neighbor(&self, direction: Direction) -> Pos {
        let mut p = *self;
        match direction {
            Direction::North => { p.y += 1; p.z -= 1 },
            Direction::NorthEast => { p.x += 1; p.z -= 1 },
            Direction::SouthEast => { p.x += 1; p.y -= 1 },
            Direction::South => { p.z += 1; p.y -= 1 },
            Direction::SouthWest => { p.z += 1; p.x -= 1 },
            Direction::NorthWest => { p.y += 1; p.x -= 1 },
        }
        p
    }

    fn fmt(&self) -> String {
        format!("{},{},{}", self.x, self.y, self.z)
    }
}

#[derive(Copy, Clone)]
struct ScreenPos {
    row: usize,
    col: usize,
}

impl ScreenPos {
    fn new(row: usize, col: usize) -> ScreenPos {
        ScreenPos {
            row: row,
            col: col,
        }
    }

    fn astuple(&self) -> (usize, usize) {
        (self.row, self.col)
    }
}

/// Representation of a Cell on screen
#[derive(Copy, Clone)]
struct ScreenCell {
    pos: Pos,
    screenpos: ScreenPos,
}

impl ScreenCell {
    fn refcell () -> ScreenCell {
        ScreenCell{
            pos: Pos::new(0, 0, 0),
            screenpos: ScreenPos::new(CELL_CENTER_ROW, CELL_CENTER_COL),
        }
    }

    fn neighbor(&self, direction: Direction) -> ScreenCell {
        let p = Pos::new(0, 0, 0).neighbor(direction);
        self.relative_cell(p)
    }

    fn relative_cell(&self, relative_pos: Pos) -> ScreenCell {
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

    fn contents_screenpos(&self, dy: i8, dx: i8) -> ScreenPos{
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

fn printline(term: &mut Terminal, screenpos: ScreenPos, line: &str) {
    for (index, ch) in line.chars().enumerate() {
        let x = screenpos.col + index;
        if x >= term.cols() {
            break;
        }
        term[(screenpos.row, x)].set_ch(ch);
    }
}

fn direction_for_key(key: char) -> Option<Direction> {
    match key {
        '8' => Some(Direction::North),
        '9' => Some(Direction::NorthEast),
        '3' => Some(Direction::SouthEast),
        '2' => Some(Direction::South),
        '1' => Some(Direction::SouthWest),
        '7' => Some(Direction::NorthWest),
        _ => None,
    }
}

fn drawgrid(term: &mut Terminal) {
    let lines: [&str; 4] = [
        " /     \\      ",
        "/       \\ _ _ ",
        "\\       /     ",
        " \\ _ _ /      ",
    ];
    let (cols, rows) = term.size();
    let colrepeatcount = cols / lines[0].len();
    for y in 0..rows-1 {
        for colrepeat in 0..colrepeatcount {
            let x = colrepeat * lines[0].len();
            let (_, lineno) = y.div_rem(&lines.len());
            let line = lines[lineno];
            printline(term, ScreenPos{ row: y, col: x }, line);
        }
    }
}

fn drawposmarkers(term: &mut Terminal) {
    let cellit = VisibleCellIterator::new(ScreenCell::refcell(), term.cols(), term.rows());
    for sc in cellit {
        printline(term, sc.contents_screenpos(0, -3), &sc.pos.fmt());
    }
}

fn drawunit(term: &mut Terminal, pos: Pos) {
    let refcell = ScreenCell::refcell();
    let sc = refcell.relative_cell(pos);
    term[sc.contents_screenpos(1, 0).astuple()].set_ch('X');
}

fn main() {
    let mut unitpos = Pos::new(0, 0, 0);
    loop {
        let mut term = Terminal::new().unwrap();
        drawgrid(&mut term);
        drawposmarkers(&mut term);
        drawunit(&mut term, unitpos);
        let _ = term.swap_buffers();
        match term.get_event(-1) {
            Ok(Some(Event::Key(k))) => {
                if k == 'q' {
                    break;
                }
                match direction_for_key(k) {
                    Some(d) => {
                        unitpos = unitpos.neighbor(d);
                    },
                    None => {},
                };
            },
            _ => { break; },
        }
    }
}

