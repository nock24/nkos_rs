use alloc::vec::Vec;

use crate::drivers::serial;

pub fn run() {
    serial::clear();
    let mut state = State::new();
    state.print_status_bar();

    loop {
        if let Some(input) = Input::poll(&state) {
            if input == Input::Quit {
                break;
            }
            input.exec(&mut state);
        }
    }

    serial::clear();
}

struct State {
    lines: Vec<Vec<u8>>,
    mode: Mode,
    cursor_pos: (usize, usize),
    term_rows: usize,
    term_cols: usize,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
}

impl State {
    fn new() -> Self {
        let (term_rows, term_cols) = serial::dimensions();
        Self {
            lines: vec![Vec::new()],
            mode: Mode::Normal,
            cursor_pos: (0, 0),
            term_rows,
            term_cols,
        }
    }

    fn print_status_bar(&self) {
        serial::move_cursor((0, self.term_rows - 1));

        serial::clear_line();
        serial::write(match self.mode {
            Mode::Normal => b"(NORMAL)",
            Mode::Insert => b"[INSERT]",
        });

        serial::move_cursor(self.cursor_pos);
    }

    fn new_line(&mut self) {
        let (x, y) = &mut self.cursor_pos;
        if *y < self.term_rows - 2 {
            serial::write(b"\n");
            *y += 1;
            *x = 0;
            self.lines.push(Vec::new());
        }
    }

    fn to_line_end(&mut self) {
        let (x, y) = &mut self.cursor_pos;
        *x = self.lines[*y].len() - 1;
        serial::move_cursor(self.cursor_pos);
    }

    fn to_line_start(&mut self) {
        self.cursor_pos.0 = 0;
        serial::move_cursor(self.cursor_pos);
    }

    fn delete_line(&mut self) {
        serial::clear_line();

        if !self.cursor_dec_y() {
            return;
        }
        let y = self.cursor_pos.1;
        let x = self.lines[y].len();
        serial::move_cursor((x, y));
        self.cursor_pos = (x, y);

        self.lines.pop().unwrap();
    }

    fn clamp_x(&mut self) {
        let (x, y) = self.cursor_pos;
        let line_len = self.lines[y].len();
        if x > line_len {
            self.cursor_pos = (line_len, y);
            serial::move_cursor(self.cursor_pos);
        }
    }

    fn cursor_inc_x(&mut self) -> bool {
        let (x, y) = &mut self.cursor_pos;
        let line_len = self.lines[*y].len();
        if line_len > 0 && *x < line_len - 1 {
            *x += 1;
            true
        } else {
            false
        }
    }

    fn cursor_right(&mut self) {
        if self.cursor_inc_x() {
            serial::cursor_right();
        }
    }

    fn cursor_dec_x(&mut self) -> bool {
        let x = &mut self.cursor_pos.0;
        if *x > 0 {
            *x -= 1;
            true
        } else {
            false
        }
    }

    fn cursor_left(&mut self) {
        if self.cursor_dec_x() {
            serial::cursor_left();
        }
    }

    fn cursor_dec_y(&mut self) -> bool {
        let y = &mut self.cursor_pos.1;
        if *y > 0 {
            *y -= 1;
            true
        } else {
            false
        }
    }

    fn cursor_up(&mut self) {
        if self.cursor_dec_y() {
            serial::cursor_up();
            self.clamp_x();
        }
    }

    fn cursor_inc_y(&mut self) -> bool {
        let y = &mut self.cursor_pos.1;
        if *y < self.lines.len() - 1 {
            *y += 1;
            true
        } else {
            false
        }
    }

    fn cursor_down(&mut self) {
        if self.cursor_inc_y() {
            serial::cursor_down();
            self.clamp_x();
        }
    }

    fn cursor_move(&mut self, direction: Direction) {
        match direction {
            Direction::Left => self.cursor_left(),
            Direction::Right => self.cursor_right(),
            Direction::Up => self.cursor_up(),
            Direction::Down => self.cursor_down(),
        }
    }

    fn type_char(&mut self, c: u8) {
        match c {
            serial::DEL | serial::BACKSPACE => {
                if self.cursor_dec_x() {
                    let (x, y) = self.cursor_pos;
                    let line = &mut self.lines[y];
                    if x == line.len() - 1 {
                        serial::backspace();
                        line.pop().unwrap();
                    } else {
                        serial::cursor_left();
                        serial::delete_char();
                        line.remove(x);
                    }
                } else {
                    self.delete_line();
                }
            }
            _ => {
                let (x, y) = &mut self.cursor_pos;
                if c == b'\n' {
                    self.new_line();
                } else if *x < self.term_cols && *x <= self.lines[*y].len() {
                    *x += 1;
                    let line = &mut self.lines[*y];

                    if *x == line.len() + 1 {
                        serial::write_char(c);
                        line.push(c);
                    } else {
                        serial::insert_char(c);
                        line.insert(*x - 1, c);
                    }
                }
            }
        }
    }

    fn delete_char(&mut self) {
        let (x, y) = self.cursor_pos;
        if x == 0 {
            self.delete_line();
        } else if x < self.lines[y].len() {
            let line = &mut self.lines[y];
            serial::delete_char();
            line.remove(x);
        }
    }

    fn print_line(&self, idx: usize) {
        serial::clear_line();
        serial::move_cursor((0, self.cursor_pos.1));

        let line = self.lines[idx].as_slice();
        serial::write(line);

        serial::move_cursor(self.cursor_pos);
    }

    fn change_mode(&mut self, mode: Mode) {
        match mode {
            Mode::Insert => serial::line_cursor(),
            _ => {
                serial::block_cursor();

                match self.mode {
                    Mode::Insert => self.cursor_left(),
                    _ => {}
                }
            }
        }

        self.mode = mode;
        self.print_status_bar();
    }

    fn insert_mode(&mut self, left: bool) {
        if !left {
            let (x, y) = &mut self.cursor_pos;
            let line_len = self.lines[*y].len();
            if *x >= line_len {
                return;
            }
            serial::cursor_right();
            *x += 1;
        }

        self.change_mode(Mode::Insert);
    }
}

#[derive(PartialEq)]
enum Input {
    /// `true`: left, `false`: right
    Insert(bool),
    Mode(Mode),
    Direction(Direction),
    LineEnd,
    LineStart,
    Delete,
    Char(u8),
    Save,
    Quit,
}

impl Input {
    fn poll(state: &State) -> Option<Self> {
        let c = serial::read_char();
        match state.mode {
            Mode::Normal => match c {
                b'h' | b'j' | b'k' | b'l' => Some(Direction::from_char(c).into()),
                b'i' => Some(Self::Insert(true)),
                b'a' => Some(Self::Insert(false)),
                b'x' => Some(Self::Delete),
                b'$' => Some(Self::LineEnd),
                b'^' => Some(Self::LineStart),
                serial::CTRL_S => Some(Self::Save),
                serial::CTRL_Q => Some(Self::Quit),
                _ => None,
            },
            Mode::Insert => match c {
                serial::CTRL_N => Some(Mode::Normal.into()),
                c if serial::is_typeable(c) => Some(Self::Char(c)),
                _ => None,
            },
        }
    }

    fn exec(self, state: &mut State) {
        match self {
            Self::Insert(left) => state.insert_mode(left),
            Self::Mode(mode) => state.change_mode(mode),
            Self::Direction(direction) => state.cursor_move(direction),
            Self::LineEnd => state.to_line_end(),
            Self::LineStart => state.to_line_start(),
            Self::Delete => state.delete_char(),
            Self::Char(c) => state.type_char(c),
            Self::Save => todo!(),
            Self::Quit => unreachable!(), // The quit input should have already been handled.
        }
    }
}

#[derive(PartialEq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn from_char(c: u8) -> Self {
        match c {
            b'h' => Self::Left,
            b'l' => Self::Right,
            b'k' => Self::Up,
            b'j' => Self::Down,
            _ => panic!("invalid direction"),
        }
    }
}

impl From<Mode> for Input {
    fn from(x: Mode) -> Self {
        Self::Mode(x)
    }
}

impl From<Direction> for Input {
    fn from(x: Direction) -> Self {
        Self::Direction(x)
    }
}
