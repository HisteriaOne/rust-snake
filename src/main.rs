extern crate termion;

// use termion::event::Key;
use termion::input::TermRead;
use std::io::{self, Read, Write};
use termion::raw::IntoRawMode;
use termion::{clear, cursor, style};

// The upper and lower boundary char.
const HORZ_BOUNDARY: &'static str = "─";
// The left and right boundary char.
const VERT_BOUNDARY: &'static str = "│";
// The top-left corner
const TOP_LEFT_CORNER: &'static str = "┌";
// The top-right corner
const TOP_RIGHT_CORNER: &'static str = "┐";
// The bottom-left corner
const BOTTOM_LEFT_CORNER: &'static str = "└";
// The bottom-right corner
const BOTTOM_RIGHT_CORNER: &'static str = "┘";

fn main() {
    // Get and lock the stdios
    let stdout = io::stdout();
    let stdout = stdout.lock();

    let stdin = io::stdin();
    let stdin = stdin.lock();

    // let stderr = io::stderr();
    // let mut stderr = stderr.lock();

    let mut stdout = stdout.into_raw_mode().unwrap();

    let termsize = termion::terminal_size().ok();
    let width = termsize.map(|(w, _)| w).unwrap_or(70);
    let height = termsize.map(|(_, h)| h).unwrap_or(30);

    write!(stdout, "{}{}", termion::clear::All, cursor::Goto(1, 1)).unwrap();
    stdout.write(TOP_LEFT_CORNER.as_bytes()).unwrap();
    for _ in 0..width - 2 {
        stdout.write(HORZ_BOUNDARY.as_bytes()).unwrap();
    }
    stdout.write(TOP_RIGHT_CORNER.as_bytes()).unwrap();
    stdout.write(b"\n\r").unwrap();
    for h in 0..height - 3 {
        write!(
            stdout,
            "{}{}{}{}\n\r",
            cursor::Goto(1, 2 + h),
            VERT_BOUNDARY,
            cursor::Goto(width, 2 + h),
            VERT_BOUNDARY
        ).unwrap();
    }
    stdout.write(BOTTOM_LEFT_CORNER.as_bytes()).unwrap();
    for _ in 0..width - 2 {
        stdout.write(HORZ_BOUNDARY.as_bytes()).unwrap();
    }
    stdout.write(BOTTOM_RIGHT_CORNER.as_bytes()).unwrap();
    stdout.write(b"\n\r").unwrap();

    let mut stdin_keys = stdin.keys();
    loop {
        let b = stdin_keys.next().unwrap().unwrap();

        use termion::event::Key::*;

        match b {
            Char('q') => return,
            _ => {}
        }
    }
}
