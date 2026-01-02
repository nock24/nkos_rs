#![allow(dead_code)]

unsafe extern "C" {
    static __bss_start: u8;
    static __bss_end: u8;

    static __heap_start: u8;
    static __heap_size: u8;
    static __heap_end: u8;
}

pub const unsafe fn bss_start() -> *mut u8 {
    unsafe { &__bss_start as *const u8 as *mut u8 }
}

pub const unsafe fn bss_end() -> *mut u8 {
    unsafe { &__bss_end as *const u8 as *mut u8 }
}

pub const unsafe fn heap_start() -> *mut u8 {
    unsafe { &__heap_start as *const u8 as *mut u8 }
}

pub unsafe fn heap_size() -> usize {
    unsafe { &__heap_size as *const u8 as usize }
}

pub const unsafe fn heap_end() -> *mut u8 {
    unsafe { &__heap_end as *const u8 as *mut u8 }
}
