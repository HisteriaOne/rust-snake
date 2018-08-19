extern crate termion;

// use std::collections::VecDeque;
use std::io::{self, Read, Write};
// use std::thread::sleep;
// use std::time::{Duration, Instant};
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
            "{}{}{}{}",
            BOTTOM_LEFT_CORNER, horz_border, BOTTOM_RIGHT_CORNER, NEWLINE
        )
    );

    out.draw(&extent.top_left, &border);
    out.update();
}

//
// enum Direction {
//     Up,
//     Down,
//     Left,
//     Right,
// }
//
// fn move_point(pt: Point, dir: Direction) -> Option<Point> {
//     match dir {
//         Direction::Up => pt - Point(0, 1),
//     }
// }
// fn move_ord(width: u16, ord: u32, dir: Direction) -> u32 {
//     let pos = ord2coord(width, ord);
//     match dir {
//         Direction::Up => coord2ord(width, pos.0, pos.1 - 1),
//         Direction::Down => coord2ord(width, pos.0, pos.1 + 1),
//         Direction::Left => coord2ord(width, pos.0 - 1, pos.1),
//         Direction::Right => coord2ord(width, pos.0 + 1, pos.1),
//     }
// }
//

// struct Snake {
//     body: VecDeque<Point>,
//     direction: Direction,
// }
//
// impl Snake {
//     fn head(&self) -> Point {
//         assert!(!self.body.is_empty());
//         self.body.back().unwrap()
//     }
//
//     fn crawl(&self) -> Self {
//         let mut new_snake = Snake {
//             body: self.body.clone(),
//             direction: self.direction,
//         };
//         //let new_head =
//     }
// }

// struct Game<R: Read, W: Write> {
//       width: u16,
//     height: u16,
//     stdin: R,
//     stdout: W,
//     rate: u16,
//     snake: Snake,
//     food: u32,
// }
//
// impl<R: Read, W: Write> Game<R, W> {
//     fn new(stdin: R, stdout: W) -> Self {
//
//         let snake = Snake {
//             head: coord2ord(width, width / 2, height / 2),
//             body: VecDeque::new(),
//             //direction: Direction::Up,
//         };
//         let food = coord2ord(width, width / 4, height / 3);
//
//         Game {
//             width: width,
//             height: height,
//             stdin: stdin,
//             stdout: stdout,
//             rate: 100,
//             snake: snake,
//             food: food,
//         }
//     }
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
//     fn move_snake(&mut self, dir: Direction) {
//         if self.snake.head == 0 {
//             return;
//         }
//         let tail = ord2coord(self.width, self.snake.tail());
//         write!(self.stdout, "{} ", cursor::Goto(tail.0, tail.1)).unwrap();
//
//         let new_head = move_ord(self.width, self.snake.head, dir);
//         if new_head == self.food {
//             let tail = self.snake.tail();
//             self.snake.body.push_back(tail);
//         }
//         if self.snake.body.contains(&new_head) {
//             self.game_over();
//         //  game over
//         } else if self.is_border(new_head) {
//             self.game_over();
//         // game over
//         } else {
//             self.snake.crawl(new_head);
//         }
//     }
//     fn game_over(&mut self) {
//         let head = self.snake.head;
//         self.snake.head = 0;
//         self.draw_symbol(" ", head);
//
//         let body = self.snake.body.clone();
//         self.snake.body.clear();
//         for part in &body {
//             self.draw_symbol(" ", *part);
//         }
//
//         let food = self.food;
//         self.food = 0;
//         self.draw_symbol(" ", food);
//
//         write!(
//             self.stdout,
//             "{}GAME OVER!",
//             cursor::Goto(self.width / 2 - 5, self.height / 2)
//         ).unwrap();
//     }
//     fn is_border(&self, ord: u32) -> bool {
//         let pos = ord2coord(self.width, ord);
//         pos.0 == 1 || pos.0 == self.width || pos.1 == 1 || pos.1 == self.height - 1
//     }
//     fn draw_food(&mut self) {
//         let food = self.food;
//         self.draw_symbol(SNAKE_FOOD, food);
//     }
//     fn draw_snake(&mut self) {
//         let head = self.snake.head;
//         self.draw_symbol(SNAKE_HEAD, head);
//         let body = self.snake.body.clone();
//         for part in &body {
//             self.draw_symbol(SNAKE_BODY, *part);
//         }
//     }
// }

fn main() {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let stdout = stdout.into_raw_mode().unwrap();

    let mut display = SymbolDisplay { device: stdout };

    let extent = ScreenExtent::from_terminal((70, 30));
    draw_border(&mut display, &extent);

    let stdin = async_stdin();
    let input = SymbolInput { device: stdin };
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
