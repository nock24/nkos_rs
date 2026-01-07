#![no_std]
#![no_main]

use core::{
    panic::PanicInfo,
    arch::asm,
};

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[macro_use]
mod macros;
mod linker_ptrs;
mod drivers;
mod heap;
mod buf_vec;
mod shell;
mod fs;

use drivers::{
    serial,
    sd,
};

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    init_drivers();

    things();

    shell::run();
}

fn init_drivers() {
    serial::println!("Initialising drivers...");
    serial::init();
    sd::init();
    serial::println!("Drivers initialised.");
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
    serial::println!("Message: {}", core::str::from_utf8(msg.as_ref()).unwrap());

    sector_buf.write().unwrap();
}

#[inline(always)]
pub fn idle() -> ! {
    loop {
        unsafe { asm!("wfe", options(nomem, nostack)); }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    let msg = info.message();
    let location = info.location().unwrap();
    let file = location.file();
    let line = location.line();

    serial::println!("\n[KERNEL PANIC]");
    serial::println!("    Reason: {}", msg);
    serial::println!("    File: {}", file);
    serial::println!("    Line: {}", line);

    idle();
}
