use alloc::vec::Vec;

use crate::{
    idle,
    drivers::serial,
};

pub fn run() -> ! {
    serial::println!("Starting shell...");

    idle();
}
