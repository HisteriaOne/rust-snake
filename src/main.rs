extern crate termion;

use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{async_stdin, cursor};

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
const SNAKE_HEAD: &'static str = "@";
const SNAKE_BODY: &'static str = "■";

fn ord2coord(width: u16, ord: u32) -> (u16, u16) {
    ((ord / width as u32) as u16, (ord % width as u32) as u16)
}
fn coord2ord(width: u16, x: u16, y: u16) -> u32 {
    x as u32 * width as u32 + y as u32
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn move_ord(width: u16, ord: u32, dir: Direction) -> u32 {
    let pos = ord2coord(width, ord);
    match dir {
        Direction::Up => coord2ord(width, pos.0, pos.1 - 1),
        Direction::Down => coord2ord(width, pos.0, pos.1 + 1),
        Direction::Left => coord2ord(width, pos.0 - 1, pos.1),
        Direction::Right => coord2ord(width, pos.0 + 1, pos.1),
    }
}

struct Snake {
    head: u32,
    body: VecDeque<u32>,
    //direction: Direction,
}

impl Snake {
    fn tail(&self) -> u32 {
        if let Some(v) = self.body.front() {
            *v
        } else {
            self.head
        }
    }
    fn crawl(&mut self, new_head: u32) {
        if !self.body.is_empty() {
            self.body.pop_front();
            self.body.push_back(self.head);
        }
        self.head = new_head;
    }
}
struct Game<R: Read, W: Write> {
    width: u16,
    height: u16,
    stdin: R,
    stdout: W,
    rate: u16,
    snake: Snake,
    food: u32,
}

impl<R: Read, W: Write> Game<R, W> {
    fn new(stdin: R, stdout: W) -> Self {
        let termsize = termion::terminal_size().ok();
        let width = termsize.map(|(w, _)| w).unwrap_or(70);
        let height = termsize.map(|(_, h)| h).unwrap_or(30);

        let snake = Snake {
            head: coord2ord(width, width / 2, height / 2),
            body: VecDeque::new(),
            //direction: Direction::Up,
        };
        let food = coord2ord(width, width / 4, height / 3);

        Game {
            width: width,
            height: height,
            stdin: stdin,
            stdout: stdout,
            rate: 100,
            snake: snake,
            food: food,
        }
    }
    fn start(&mut self) {
        self.reset();

        let mut before = Instant::now();

        loop {
            self.draw_food();
            self.draw_snake();
            self.update();

            let interval = 1000 / self.rate as u64;
            let now = Instant::now();
            let dt = (now.duration_since(before).subsec_nanos() / 1_000_000) as u64;
            if dt < interval {
                sleep(Duration::from_millis(interval - dt));
                continue;
            }
            before = now;

            let mut buffer: Vec<u8> = vec![];
            if let Ok(_) = self.stdin.read_to_end(&mut buffer) {
                match buffer.keys().last() {
                    Some(Ok(Key::Char('q'))) => return,
                    Some(Ok(Key::Up)) => self.move_snake(Direction::Up),
                    Some(Ok(Key::Down)) => self.move_snake(Direction::Down),
                    Some(Ok(Key::Left)) => self.move_snake(Direction::Left),
                    Some(Ok(Key::Right)) => self.move_snake(Direction::Right),
                    _ => {}
                }
            }
        }
    }
    fn move_snake(&mut self, dir: Direction) {
        if self.snake.head == 0 {
            return;
        }
        let tail = ord2coord(self.width, self.snake.tail());
        write!(self.stdout, "{} ", cursor::Goto(tail.0, tail.1)).unwrap();

        let new_head = move_ord(self.width, self.snake.head, dir);
        if new_head == self.food {
            let tail = self.snake.tail();
            self.snake.body.push_back(tail);
        }
        if self.snake.body.contains(&new_head) {
            self.game_over();
        //  game over
        } else if self.is_border(new_head) {
            self.game_over();
        // game over
        } else {
            self.snake.crawl(new_head);
        }
    }
    fn game_over(&mut self) {
        let head = self.snake.head;
        self.snake.head = 0;
        self.draw_symbol(" ", head);

        let body = self.snake.body.clone();
        self.snake.body.clear();
        for part in &body {
            self.draw_symbol(" ", *part);
        }

        let food = self.food;
        self.food = 0;
        self.draw_symbol(" ", food);

        write!(
            self.stdout,
            "{}GAME OVER!",
            cursor::Goto(self.width / 2 - 5, self.height / 2)
        ).unwrap();
    }
    fn is_border(&self, ord: u32) -> bool {
        let pos = ord2coord(self.width, ord);
        pos.0 == 1 || pos.0 == self.width || pos.1 == 1 || pos.1 == self.height - 1
    }
    fn draw_symbol(&mut self, symb: &str, ord: u32) {
        if ord > 0 {
            let pos = ord2coord(self.width, ord);
            write!(self.stdout, "{}{}", cursor::Goto(pos.0, pos.1), symb).unwrap();
        }
    }
    fn draw_food(&mut self) {
        let food = self.food;
        self.draw_symbol(SNAKE_FOOD, food);
    }
    fn draw_snake(&mut self) {
        let head = self.snake.head;
        self.draw_symbol(SNAKE_HEAD, head);
        let body = self.snake.body.clone();
        for part in &body {
            self.draw_symbol(SNAKE_BODY, *part);
        }
    }
    fn update(&mut self) {
        self.stdout.flush().unwrap();
    }
    fn reset(&mut self) {
        write!(self.stdout, "{}{}", termion::clear::All, cursor::Goto(1, 1)).unwrap();

        self.stdout.write(TOP_LEFT_CORNER.as_bytes()).unwrap();
        for _ in 0..self.width - 2 {
            self.stdout.write(HORZ_BOUNDARY.as_bytes()).unwrap();
        }
        self.stdout.write(TOP_RIGHT_CORNER.as_bytes()).unwrap();
        self.stdout.write(b"\n\r").unwrap();

        for h in 2..self.height - 1 {
            write!(
                self.stdout,
                "{}{}{}{}\n\r",
                cursor::Goto(1, h),
                VERT_BOUNDARY,
                cursor::Goto(self.width, h),
                VERT_BOUNDARY
            ).unwrap();
        }

        self.stdout.write(BOTTOM_LEFT_CORNER.as_bytes()).unwrap();
        for _ in 0..self.width - 2 {
            self.stdout.write(HORZ_BOUNDARY.as_bytes()).unwrap();
        }
        self.stdout.write(BOTTOM_RIGHT_CORNER.as_bytes()).unwrap();
        self.stdout.write(b"\n\r").unwrap();
    }
}

fn main() {
    // Get and lock the stdios
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let stdout = stdout.into_raw_mode().unwrap();

    let stdin = async_stdin();

    Game::new(stdin, stdout).start();
}
