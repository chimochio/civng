use num::integer::Integer;
use rustty::{Terminal};

extern crate num;
extern crate rustty;

const CELL_WIDTH: usize = 7;
const CELL_HEIGHT: usize = 4;

#[derive(Copy, Clone)]
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
    fn fmt(&self) -> String {
        format!("{},{},{}", self.x, self.y, self.z)
    }
}

#[derive(Copy, Clone)]
struct ScreenPos {
    row: usize,
    col: usize,
}

/// Representation of a Cell on screen
#[derive(Copy, Clone)]
struct ScreenCell {
    pos: Pos,
    screenpos: ScreenPos,
}

impl ScreenCell {
    fn neighbor(&self, direction: Direction) -> ScreenCell {
        let mut p = Pos { x: 0, y: 0, z: 0 };
        match direction {
            Direction::North => { p.y += 1; p.z -= 1 },
            Direction::NorthEast => { p.x += 1; p.z -= 1 },
            Direction::SouthEast => { p.x += 1; p.y -= 1 },
            Direction::South => { p.z += 1; p.y -= 1 },
            Direction::SouthWest => { p.z += 1; p.x -= 1 },
            Direction::NorthWest => { p.y += 1; p.x -= 1 },
        }
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

fn printline(term: &mut Terminal, screenpos: ScreenPos, line: &str) {
    for (index, ch) in line.chars().enumerate() {
        let x = screenpos.col + index;
        if x >= term.cols() {
            break;
        }
        term[(screenpos.row, x)].set_ch(ch);
    }
}

fn inside_bounds(term: &Terminal, screenpos: ScreenPos) -> bool {
    let (cols, rows) = term.size();
    screenpos.row < rows && screenpos.col < cols
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
    let (cols, rows) = term.size();
    let cellcountx = cols / 7;
    let cellcounty = rows / 4;
    let screenx = ((cellcountx / 2) * 7) + 4;
    let screeny = ((cellcounty / 2) * 4) + 1;
    let sc = ScreenCell{
        pos: Pos{ x: 0, y: 0, z: 0},
        screenpos: ScreenPos{ row: screeny, col: screenx },
    };
    printline(term, sc.contents_screenpos(0, -3), &sc.pos.fmt());
    for direction1 in [Direction::NorthEast, Direction::SouthEast, Direction::SouthWest, Direction::NorthWest, Direction::South, Direction::North].iter() {
        let mut sc2 = sc;
        while inside_bounds(&term, sc2.screenpos) {
            sc2 = sc2.neighbor(*direction1);
            printline(term, sc2.contents_screenpos(0, -3), &sc2.pos.fmt());
        }
    }
}

fn main() {
    let mut term = Terminal::new().unwrap();
    drawgrid(&mut term);
    drawposmarkers(&mut term);
    let _ = term.swap_buffers();
    let _ = term.get_event(-1);
}

