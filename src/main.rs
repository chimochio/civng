use num::integer::Integer;
use rustty::{Terminal};

extern crate num;
extern crate rustty;

fn drawgrid(term: &mut Terminal) {
    let lines: [&str; 4] = [
        " /     \\      ",
        "/       \\ _ _ ",
        "\\       /     ",
        " \\ _ _ /      ",
    ];
    let (cols, rows) = term.size();
    for y in 0..rows-1 {
        for x in 0..cols-1 {
            let (_, lineno) = y.div_rem(&lines.len());
            let line = lines[lineno];
            let (_, charno) = x.div_rem(&line.len());
            term[y][x].set_ch(line.as_bytes()[charno] as char);
        }
    }
}

fn main() {
    let mut term = Terminal::new().unwrap();
    drawgrid(&mut term);
    let _ = term.swap_buffers();
    let _ = term.get_event(-1);
}

