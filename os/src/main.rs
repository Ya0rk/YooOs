#![no_std]
#![no_main]
// #[macro_use]

use core::arch::global_asm;

mod sbi;
mod trap;
mod myutil;

global_asm!(include_str!("entry.asm"));

/// the rust entry-point of os
#[no_mangle]
pub fn rust_main() -> ! {
    myutil::booting::clear_bss();
    trap::init();
    myutil::booting::show_logo();
    
    myutil::logging::init();
    println!("[kernel] Hello, world!");

    sbi::shutdown(false)
}
