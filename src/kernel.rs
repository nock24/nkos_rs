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
//mod fs;

use drivers::{
    serial,
    sd,
};

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    init_drivers();

    set_msg();

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
        msg: [u8; 12],
    }
}

fn set_msg() {
    let msg = b"hello world!";

    let mut buf = sd::DynSectorBuf::new(0, 1);
    buf.read().unwrap();

    BootSector::msg_write(buf.as_mut_dyn_buf(..), msg);

    buf.write().unwrap();
}

fn things() {
    let mut buf = sd::DynSectorBuf::new(0, 1);
    buf.read().unwrap();

    let boot_cnt = BootSector::boot_cnt(buf.as_dyn_buf(..));
    BootSector::set_boot_cnt(buf.as_mut_dyn_buf(..), boot_cnt + 1);

    let mut msg = [0; 12];
    BootSector::msg_read(buf.as_dyn_buf(..), &mut msg);
    serial::println!("Message: {}", core::str::from_utf8(&msg).unwrap());

    buf.write().unwrap();
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
