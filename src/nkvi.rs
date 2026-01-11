use crate::drivers::serial;

pub fn run() -> ! {
    serial::clear();

    let mut state = State::new();

    loop {
        state.print_status_bar();

        if let Some(input) = Input::poll(&state) {
            input.exec(&mut state);
        }
    }
}

struct State {
    mode: Mode,
    cursor_pos: (usize, usize),
    term_rows: usize,
    term_cols: usize,
}

enum Mode {
    Normal,
    Insert,
}

impl State {
    fn new() -> Self {
        let (term_rows, term_cols) = serial::dimensions();
        Self {
            mode: Mode::Normal,
            cursor_pos: (0, 0),
            term_rows,
            term_cols,
        }
    }

    fn print_status_bar(&self) {
        serial::move_cursor((self.term_rows, 1));

        serial::write(match self.mode {
            Mode::Normal => b"(NORMAL)",
            Mode::Insert => b"[INSERT]",
        });
        serial::print!("  cursor pos: {:?}", self.cursor_pos);

        let (x, y) = self.cursor_pos;
        serial::move_cursor((y + 1, x + 1));
    }

    fn cursor_right(&mut self) {
        let x = &mut self.cursor_pos.0;
        if *x != self.term_cols - 1 {
            *x += 1;
            serial::cursor_right();
        }
    }

    fn cursor_left(&mut self) {
        let x = &mut self.cursor_pos.0;
        if *x != 0 {
            *x -= 1;
            serial::cursor_left();
        }
    }

    fn cursor_up(&mut self) {
        let y = &mut self.cursor_pos.1;
        if *y != 0 {
            *y -= 1;
            serial::cursor_up();
        }
    }

    fn cursor_down(&mut self) {
        let y = &mut self.cursor_pos.1;
        if *y != self.term_rows - 1 {
            *y += 1;
            serial::cursor_down();
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

    fn type_char(&mut self, ch: u8) {
        match ch {
            serial::DEL | serial::BACKSPACE => {
                let x = &mut self.cursor_pos.0;
                if *x != 0 {
                    *x -= 1;
                    serial::backspace();
                }
            }
            _ => {
                if ch == b'\n' {
                    serial::write_char(b'\r');
                }
                let x = &mut self.cursor_pos.0;
                if *x != self.term_cols - 1 {
                    *x += 1;
                    serial::write_char(ch);
                }
            }
        }
    }

    fn change_mode(&mut self, mode: Mode) {
        self.mode = mode;
        //self.print_status_bar();
    }
}

enum Input {
    Mode(Mode),
    Move(Direction),
    Char(u8),
}

const CTRL_N: u8 = 0x0E;

impl Input {
    fn poll(state: &State) -> Option<Self> {
        let ch = serial::read_char();
        match state.mode {
            Mode::Normal => match ch {
                b'h' | b'j' | b'k' | b'l' => Some(Direction::from_char(ch).into()),
                b'i' => Some(Mode::Insert.into()),
                _ => None,
            },
            Mode::Insert => match ch {
                CTRL_N => Some(Mode::Normal.into()),
                _ => Some(Self::Char(ch)),
            },
        }
    }

    fn exec(self, state: &mut State) {
        match self {
            Self::Mode(mode) => state.change_mode(mode),
            Self::Move(direction) => state.cursor_move(direction),
            Self::Char(ch) => state.type_char(ch),
        }
    }
}

enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn from_char(ch: u8) -> Self {
        match ch {
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
        Self::Move(x)
    }
}
