use super::{cmd::Cmd, cmd_parser};
use crate::drivers::serial;

pub fn run() -> ! {
    sprintln!("Starting shell...");

    loop {
        sprint!("guest@nkos [~] $ ");

        let str = serial::read_line_utf8();
        let cmd = cmd_parser::parse(str.as_slice());
        match cmd {
            Ok(cmd) => cmd.run(),
            Err(msg) if msg.len() > 0 => sprintln!("{}", msg),
            _ => {}
        }
    }
}
