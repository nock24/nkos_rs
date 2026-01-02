use crate::{
    shell::command,
    drivers::serial,
};

pub fn run() -> ! {
    serial::println!("Starting shell...");

    loop {
        serial::print!("guest@nkos [~] $ ");

        let str = serial::read_line_utf8();
        let cmd = command::parse(&str);
        handle_cmd_result(cmd);
    }
}

fn handle_cmd_result<'a>(result: command::Result) {
    match result {
        Ok(cmd) => cmd.run(),
        Err(msg) if msg.len() > 0 => serial::println!("{}", msg),
        _ => {},
    }
}
