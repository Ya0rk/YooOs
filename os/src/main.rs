#![no_std]
#![no_main]

use core::arch::global_asm;

#[macro_use]
mod console;
mod lang_items;
mod logging;
mod sbi;
mod booting;

global_asm!(include_str!("entry.asm"));

/// the rust entry-point of os
#[no_mangle]
pub fn rust_main() -> ! {
    booting::clear_bss();
    booting::show_logo();
    
    logging::init();
    println!("[kernel] Hello, world!");

    sbi::shutdown(false)
}
