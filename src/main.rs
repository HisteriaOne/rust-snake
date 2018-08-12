extern crate termion;

// use termion::event::Key;
use std::io::{self, Read, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{async_stdin, clear, cursor, style};

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

const SNAKE_FOOD: &'static str = "🐭";
const SNAKE_HEAD: &'static str = "■";
const SNAKE_BODY: &'static str = "□";

struct Snake {
    head: usize,
    body: Vec<usize>,
}

struct Game<R: Read, W: Write> {
    width: usize,
    height: usize,
    stdin: R,
    stdout: W,
    rate: usize,
}

impl<R: Read, W: Write> Game<R, W> {
    fn start(&mut self) {
        //        clear();
        //
        //        init_snake();
        //        init_food();
        //
        //        draw_border();
        //
        //        loop{
        //            let input = 8;
        //        }
        //        draw_snake();
        //        draw_food();
        //
    }
}

fn main() {
    // Get and lock the stdios
    let stdout = io::stdout();
    let stdout = stdout.lock();

    let mut stdin = async_stdin();

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

    write!(stdout, "{}", cursor::Goto(43, 12)).unwrap();
    write!(stdout, "{}", SNAKE_FOOD).unwrap();

    let mut pos = (50, 12);
    //    let mut keys = stdin.keys();
    let mut last = Instant::now();
    loop {
        let mut buf = [0];
        stdin.read(&mut buf).unwrap();

        match buf[0] {
            b'q' => return,
            _ => {}
        }
        stdout.flush().unwrap();


        write!(stdout, "{}", cursor::Goto(pos.0, pos.1)).unwrap();
        write!(stdout, "{}{}{}", SNAKE_HEAD, SNAKE_BODY, SNAKE_BODY).unwrap();
        stdout.flush().unwrap();

        let now = Instant::now();
        let dt = (now.duration_since(last).subsec_nanos() / 1_000_000) as u64;
        let duration = 300;
        if dt < duration {
            sleep(Duration::from_millis(duration - dt));
            continue;
        }
        last = now;

        pos.0 -= 1;
        if pos.0 == 0 {
            pos.0 = width - 10;
        }
    }
}
