use crate::drivers::serial;

pub fn run() -> ! {
    let mut state = State::new();
    loop {
        if let Some(input) = Input::poll(&state) {
            input.exec(&mut state);
        }
    }
}

struct State {
    mode: Mode,
}

enum Mode {
    Normal,
    Insert,
}

impl State {
    fn new() -> Self {
        Self { mode: Mode::Normal }
    }
}

enum Input {
    Mode(Mode),
    Move(Direction),
    Char(u8),
}

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
                b'n' => Some(Mode::Normal.into()),
                _ => Some(Self::Char(ch)),
            },
        }
    }

    fn exec(self, state: &mut State) {
        match self {
            Self::Mode(mode) => state.mode = mode,
            Self::Move(direction) => direction.move_cursor(),
            Self::Char(ch) => {
                if ch == b'\n' {
                    serial::write_char(b'\r');
                }
                serial::write_char(ch);
            }
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

    fn move_cursor(self) {
        match self {
            Self::Left => serial::write_char(serial::BACKSPACE),
            Self::Right => todo!(),
            Self::Up => todo!(),
            Self::Down => serial::write_char(b'\n'),
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
