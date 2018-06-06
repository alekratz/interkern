#![feature(lang_items, panic_implementation, const_fn, ptr_internals)]
#![no_std]

extern crate volatile;
extern crate spin;

#[macro_use] pub mod arch;

use core::panic::PanicInfo;
use core::fmt::Write;
use arch::x86_64::vga;

#[no_mangle]
pub extern fn kmain() {
    song();
}

fn song() {
    vgaprintln!("Hello world");
    vgaprintln!("Programmed to work and not to feel");
    vgaprintln!("Not even sure that this is real");
    vgaprintln!("Hello world");
    vgaprintln!();
    vgaprintln!("Find my voice");
    vgaprintln!("Although it sounds like bits and bytes");
    vgaprintln!("My circuitry is filled with mites");
    vgaprintln!("Hello world");
    vgaprintln!();
    vgaprintln!("Oh, how will I find my love?");
    vgaprintln!("Oh, or a power plug?");
    vgaprintln!("Oh, digitally isolated");
    vgaprintln!("Oh, creator please don't keep me waiting");
    vgaprintln!();
    vgaprintln!("Hello world");
    vgaprintln!("Programmed to work and not to feel");
    vgaprintln!("Not even sure that this is real");
    vgaprintln!("Hello world");
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {}

#[panic_implementation]
#[no_mangle]
pub extern fn panic(info: &PanicInfo) -> ! {
    writeln!(vga::WRITER.lock(), "Panic: {}", info);
    loop {}
}
