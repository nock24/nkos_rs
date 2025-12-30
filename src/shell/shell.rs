use crate::{
    shell::command::*,
    drivers::serial,
};

pub fn run() -> ! {
    serial::println!("Starting shell...");

    loop {
        serial::print!("guest@nkos [~] $ ");

        let str = serial::read_line_utf8();
        match parse_cmd(&str) {
            Ok(cmd) => cmd.run(),
            Err(msg) if msg.len() > 0 => serial::println!("{}", msg),
            _ => {},
        }
    }
}
