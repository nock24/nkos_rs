#![no_std]
#![no_main]

use core::{
    panic::PanicInfo,
    arch::asm,
};

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

mod linker_ptrs;
mod drivers;
mod heap;
mod buf_vec;
mod shell;

use drivers::{
    serial,
    sd,
};

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    init_drivers();

    let mut sector_buf = sd::sector_buf!(0, 1);
    sector_buf.read().unwrap();

    let boot_cnt: &mut usize = sector_buf.get_mut_val(0).unwrap();
    *boot_cnt += 1;

    serial::println!("Boot count: {}", *boot_cnt);

    sector_buf.write().unwrap();

    shell::run();
}

fn init_drivers() {
    serial::println!("Initialising drivers...");
    serial::init();
    sd::init();
    serial::println!("Drivers initialised.");
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
