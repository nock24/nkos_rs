use crate::drivers::serial;
use super::command;

pub fn run() -> ! {
    serial::println!("Starting shell...");

    loop {
        serial::print!("guest@nkos [~] $ ");

        let str = serial::read_line_utf8();
        let cmd = command::parse(str.as_slice());
        match cmd {
            Ok(cmd) => cmd.run(),
            Err(msg) if msg.len() > 0 => serial::println!("{}", msg),
            _ => {},
        }
    }
}
