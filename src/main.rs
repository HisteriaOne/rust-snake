extern crate termion;

use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{async_stdin, cursor};

//
// const SNAKE_FOOD: &'static str = "□";
// const SNAKE_HEAD: &'static str = "@";
// const SNAKE_BODY: &'static str = "■";

type Coordinate = u16;
type Point = (Coordinate, Coordinate);

#[inline]
fn checked_add(lhs: &Point, rhs: &Point) -> Option<Point> {
    match (lhs.0.checked_add(rhs.0), lhs.1.checked_add(rhs.1)) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    }
}

#[inline]
fn checked_sub(lhs: &Point, rhs: &Point) -> Option<Point> {
    match (lhs.0.checked_sub(rhs.0), lhs.1.checked_sub(rhs.1)) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    }
}

struct ScreenExtent {
    top_left: Point,
    bottom_right: Point,
    width: Coordinate,
    height: Coordinate,
}

impl ScreenExtent {
    fn from_terminal(default: (Coordinate, Coordinate)) -> Self {
        let termsize = termion::terminal_size().ok();
        let width = termsize.map(|(w, _)| w).unwrap_or(default.0);
        let height = termsize.map(|(_, h)| h).unwrap_or(default.1);
        ScreenExtent::new(width, height)
    }

    fn new(width: Coordinate, height: Coordinate) -> Self {
        assert!(width > 1);
        assert!(height > 1);
        ScreenExtent {
            top_left: (1, 1),
            bottom_right: (width, height),
            width: width,
            height: height,
        }
    }

    fn contains(&self, pt: &Point) -> bool {
        pt.0 > self.top_left.0 && pt.0 < self.bottom_right.0 && pt.1 > self.top_left.1
            && pt.1 < self.bottom_right.1
    }
}

trait Draw {
    fn draw(&mut self, pos: &Point, data: &str);
    fn clear(&mut self);
    fn update(&mut self);
}

struct SymbolDisplay<W: Write> {
    device: W,
}

impl<W: Write> Draw for SymbolDisplay<W> {
    fn draw(&mut self, pos: &Point, data: &str) {
        write!(self.device, "{}{}", cursor::Goto(pos.0, pos.1), data).unwrap();
    }
    fn clear(&mut self) {
        write!(self.device, "{}{}", termion::clear::All, cursor::Goto(1, 1)).unwrap();
        self.update();
    }
    fn update(&mut self) {
        self.device.flush().unwrap();
    }
}

trait Input {
    fn last(&mut self) -> Option<Key>;
}

struct SymbolInput<R: Read> {
    device: R,
}

impl<R: Read> Input for SymbolInput<R> {
    fn last(&mut self) -> Option<Key> {
        let mut buffer: Vec<u8> = vec![];
        match self.device.read_to_end(&mut buffer) {
            Ok(_) => match buffer.keys().last() {
                Some(Ok(key)) => Some(key),
                _ => None,
            },
            Err(_) => None,
        }
    }
}

fn draw_border(out: &mut Draw, extent: &ScreenExtent) {
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
    const NEWLINE: &'static str = "\n\r";

    let inner_width = extent.width as usize - 2;
    let inner_height = extent.height as usize - 2;

    let horz_border = &HORZ_BOUNDARY.repeat(inner_width);
    let inner_line = &" ".repeat(inner_width);

    let border = format!(
        "{}{}{}",
        format!(
            "{}{}{}{}",
            TOP_LEFT_CORNER, horz_border, TOP_RIGHT_CORNER, NEWLINE
        ),
        format!(
            "{}{}{}{}",
            VERT_BOUNDARY, inner_line, VERT_BOUNDARY, NEWLINE
        ).repeat(inner_height),
        format!(
            "{}{}{}",
            BOTTOM_LEFT_CORNER, horz_border, BOTTOM_RIGHT_CORNER
        )
    );

    out.draw(&extent.top_left, &border);
    out.update();
}

#[derive(Debug)]
enum GameState {
    Begin,
    InGame(Option<Key>),
    GameOver,
    Quit,
}

fn game_state_transition(state: &GameState, input: &mut Input) -> GameState {
    let key = input.last();

    match state {
        GameState::Begin => GameState::InGame(key),
        GameState::InGame(_) => match key {
            Some(Key::Char('q')) => GameState::Quit,
            _ => GameState::InGame(key),
        },
        GameState::GameOver => match key {
            Some(Key::Char('b')) => GameState::Begin,
            Some(Key::Char('q')) => GameState::Quit,
            _ => GameState::GameOver,
        },
        GameState::Quit => GameState::Quit,
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn key_to_direction(key: Option<Key>) -> Option<Direction> {
    match key {
        Some(Key::Up) => Some(Direction::Up),
        Some(Key::Down) => Some(Direction::Down),
        Some(Key::Left) => Some(Direction::Left),
        Some(Key::Right) => Some(Direction::Right),
        _ => None,
    }
}

fn move_point(pt: &Point, dir: &Direction) -> Option<Point> {
    match dir {
        Direction::Up => checked_sub(&pt, &(0, 1)),
        Direction::Down => checked_add(&pt, &(0, 1)),
        Direction::Left => checked_sub(&pt, &(1, 0)),
        Direction::Right => checked_add(&pt, &(1, 0)),
    }
}

#[derive(Debug)]
struct Snake {
    body: VecDeque<Point>,
    direction: Direction,
}

impl Snake {
    fn new(head: Point, direction: Direction) -> Self {
        let mut body = VecDeque::new();
        body.push_back(head);
        Snake { body, direction }
    }
    fn update(&self, direction: Option<Direction>) -> Snake {
        Snake {
            body: self.body.clone(),
            direction: match direction {
                Some(Direction::Up) | Some(Direction::Down) => match self.direction {
                    Direction::Left | Direction::Right => direction.unwrap(),
                    _ => self.direction,
                },
                Some(Direction::Left) | Some(Direction::Right) => match self.direction {
                    Direction::Up | Direction::Down => direction.unwrap(),
                    _ => self.direction,
                },
                _ => self.direction,
            },
        }.crawl()
    }

    fn crawl(&self) -> Self {
        let direction = self.direction;
        let mut body = self.body.clone();
        assert!(!body.is_empty());
        if let Some(new_head) = move_point(body.back().unwrap(), &direction) {
            body.push_back(new_head);
            body.pop_front().unwrap();
        }
        Snake { body, direction }
    }

    fn head(&self) -> &Point {
        assert!(!self.body.is_empty());
        return self.body.back().unwrap();
    }

    fn eat(&self, food: &Point) -> Option<Snake> {
        if self.head() == food {
            let mut body = self.body.clone();
            body.push_front(*self.body.front().unwrap());
            return Some(Snake {
                body,
                direction: self.direction,
            });
        } else {
            None
        }
    }
    fn self_cross(&self) -> bool {
        let head = self.head();
        self.body.len() != 1 && self.body.iter().filter(|&x| x == head).count() != 1
    }
}

trait Drawable {
    fn draw(&self, output: &mut Draw);
    fn clear(&self, output: &mut Draw);
}

impl Drawable for Snake {
    fn clear(&self, output: &mut Draw) {
        for x in &self.body {
            output.draw(&x, " ");
        }
    }
    fn draw(&self, output: &mut Draw) {
        for x in &self.body {
            output.draw(&x, "@");
        }
    }
}

struct Game<I: Input, D: Draw> {
    screen: ScreenExtent,
    input: I,
    output: D,
    state: GameState,
    snake: Snake,
    food: Point,
}

impl<I: Input, D: Draw> Game<I, D> {
    fn new(input: I, output: D, screen: ScreenExtent) -> Self {
        let snake = Snake::new((screen.width / 2, screen.height / 2), Direction::Up);
        let food = (screen.width / 2, screen.height / 2 - 2);
        Game {
            screen: screen,
            input: input,
            output: output,
            state: GameState::Begin,
            snake: snake,
            food: food,
        }
    }

    fn run(&mut self) {
        loop {
            match self.state {
                GameState::Begin => {
                    self.output.clear();
                    draw_border(&mut self.output, &self.screen);
                    self.output.update();
                }
                GameState::InGame(key) => {
                    self.snake.clear(&mut self.output);
                    self.snake = self.snake.update(key_to_direction(key));

                    // self.output.draw(
                    //     &self.screen.top_left,
                    //     &format!("{}{:?}", termion::clear::CurrentLine, &self.snake.body),
                    // );
                    if !self.screen.contains(&self.snake.head()) {
                        self.state = GameState::GameOver;
                    } else if self.snake.self_cross() {
                        self.state = GameState::GameOver;
                    } else {
                        if let Some(snake) = self.snake.eat(&self.food) {
                            self.snake = snake;
                            self.food = (self.food.1, self.food.0);
                        }
                        self.snake.draw(&mut self.output);
                        self.output.draw(&self.food, "X");
                        self.output.update();
                    }
                }
                GameState::GameOver => {
                    self.output.clear();
                    self.output.draw(
                        &(self.screen.width / 2 - 5, self.screen.height / 2),
                        "Game Over",
                    );
                    self.output.update();
                }
                GameState::Quit => {
                    self.output.clear();
                    return;
                }
            }
            self.state = game_state_transition(&self.state, &mut self.input);
            sleep(Duration::from_millis(300));
        }
    }
}
//     fn start(&mut self) {
//         self.reset();
//
//         let mut before = Instant::now();
//
//         loop {
//             self.draw_food();
//             self.draw_snake();
//             self.update();
//
//             let interval = 1000 / self.rate as u64;
//             let now = Instant::now();
//             let dt = (now.duration_since(before).subsec_nanos() / 1_000_000) as u64;
//             if dt < interval {
//                 sleep(Duration::from_millis(interval - dt));
//                 continue;
//             }
//             before = now;
//
//             let mut buffer: Vec<u8> = vec![];
//             if let Ok(_) = self.stdin.read_to_end(&mut buffer) {
//                 match buffer.keys().last() {
//                     Some(Ok(Key::Char('q'))) => return,
//                     Some(Ok(Key::Up)) => self.move_snake(Direction::Up),
//                     Some(Ok(Key::Down)) => self.move_snake(Direction::Down),
//                     Some(Ok(Key::Left)) => self.move_snake(Direction::Left),
//                     Some(Ok(Key::Right)) => self.move_snake(Direction::Right),
//                     _ => {}
//                 }
//             }
//         }
//     }

fn main() {
    let stdin = async_stdin();

    let stdout = io::stdout();
    let stdout = stdout.lock();
    let stdout = stdout.into_raw_mode().unwrap();

    let extent = ScreenExtent::from_terminal((70, 30));

    Game::new(
        SymbolInput { device: stdin },
        SymbolDisplay { device: stdout },
        extent,
    ).run();
}

#[cfg(test)]
mod tests {
    // use checked_add;
    use checked_sub;
    #[test]
    fn point_sub() {
        assert_eq!(checked_sub(&(1, 1), &(0, 1)), Some((1, 0)));
        assert_eq!(checked_sub(&(1, 0), &(0, 1)), None);
        assert_eq!(checked_sub(&(1, 1), &(1, 0)), Some((0, 1)));
        assert_eq!(checked_sub(&(0, 1), &(1, 0)), None);
    }

    use ScreenExtent;
    #[test]
    fn screen_contains() {
        let screen = ScreenExtent::new(5, 8);

        assert!(!screen.contains(&(1, 1)));
        assert!(!screen.contains(&(1, 2)));
        assert!(!screen.contains(&(1, 8)));
        assert!(!screen.contains(&(1, 9)));

        assert!(!screen.contains(&(2, 1)));
        assert!(screen.contains(&(2, 2)));
        assert!(screen.contains(&(2, 7)));
        assert!(!screen.contains(&(2, 8)));

        assert!(!screen.contains(&(4, 1)));
        assert!(screen.contains(&(4, 2)));
        assert!(screen.contains(&(4, 7)));
        assert!(!screen.contains(&(4, 8)));

        assert!(!screen.contains(&(5, 2)));
        assert!(!screen.contains(&(3, 8)));
    }
}
