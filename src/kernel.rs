#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

mod buf_vec;
#[macro_use]
mod drivers;
mod fs;
mod heap;
mod linker_ptrs;
#[macro_use]
mod macros;
mod nkvi;
mod shell;

use drivers::{sd, serial};
use fs::file::{File, TextFile};

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    init_drivers();

    things();

    /*
    let mut file = TextFile::new(2).unwrap();
    file.read().expect("failed to read file");
    let str = file.str().unwrap();
    serial::println!("File contents: {}", str);
    let str = file.mut_str().unwrap();
    str.push(b'a');
    file.write().expect("failed to write file");
    */

    shell::run();
}

fn init_drivers() {
    serial::init();
    sd::init();
    sprintln!("Drivers initialised.");
}

sd::sector_layout! {
    pub BootSector {
        boot_cnt: u32,
        msg_len: u8,
        msg: [u8; msg_len],
    }
}

fn set_msg(msg: &[u8]) {
    let mut sector_buf = sd::SectorBuf::new(0, 1);
    sector_buf.read().unwrap();

    BootSector::set_msg_len(sector_buf.as_mut_buf(..), msg.len() as u8);
    BootSector::msg_write(sector_buf.as_mut_buf(..), msg);

    sector_buf.write().unwrap();
}

fn things() {
    let mut sector_buf = sd::SectorBuf::new(0, 1);
    sector_buf.read().unwrap();

    let boot_cnt = BootSector::boot_cnt(sector_buf.as_buf(..));
    BootSector::set_boot_cnt(sector_buf.as_mut_buf(..), boot_cnt + 1);

    let msg = BootSector::msg_boxed(sector_buf.as_buf(..));
    sprintln!("Message: {}", core::str::from_utf8(msg.as_ref()).unwrap());

    sector_buf.write().unwrap();
}

#[inline(always)]
pub fn idle() -> ! {
    loop {
        unsafe {
            asm!("wfe", options(nomem, nostack));
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    let msg = info.message();
    let location = info.location().unwrap();
    let file = location.file();
    let line = location.line();

    sprintln!("\n[KERNEL PANIC]");
    sprintln!("    Reason: {}", msg);
    sprintln!("    File: {}", file);
    sprintln!("    Line: {}", line);

    idle();
}
