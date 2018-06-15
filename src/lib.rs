#![feature(lang_items, panic_implementation, ptr_internals)]
#![feature(const_fn, const_let)]
#![feature(alloc, allocator_api, global_allocator)]
#![feature(abi_x86_interrupt)]
#![feature(nll)]
#![no_std]

extern crate rlibc;
#[macro_use] extern crate alloc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;
#[macro_use] extern crate bitflags;
extern crate x86_64;
#[macro_use] extern crate static_assertions;
#[macro_use] extern crate lazy_static;
extern crate bit_field;

#[macro_use] pub mod arch;
pub mod memory;

use core::panic::PanicInfo;
use memory::BuddyAllocator;

#[link_section = ".data"]
#[cfg(not(test))]
#[global_allocator]
pub static GLOBAL_ALLOCATOR: BuddyAllocator = memory::KERNEL_HEAP_ALLOCATOR;

/// The kernel entrypoint.
///
/// # Arguments
/// `boot_info_addr` - the address where multiboot2 system information resides.
#[cfg(not(test))]
#[no_mangle]
pub extern fn kmain(boot_info_addr: usize) {
    let boot_info = unsafe { multiboot2::load(boot_info_addr) };
    // TODO(arch) this is x86_64 specific
    arch::x86_64::enable_nxe_bit();
    arch::x86_64::enable_kernel_write_protect();

    let mut memory_controller = memory::init(boot_info);
    arch::x86_64::interrupt::init(&mut memory_controller);
    //x86_64::instructions::interrupts::int3();

    vgaprintln!();
    vgaprintln!("================================================================================");
    vgaprintln!();

    //welcome();

    let my_vec = vec!(vec!(22; 90); 90);
    /*
    for b in my_vec.into_iter() {
        vgaprintln!("raw: {:?}", <::alloc::boxed::Box<_>>::into_raw(b));
    }
    */
    vgaprintln!("Testing vector heap");
    vgaprintln!("Done");
}

fn welcome() {
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
    vgaprintln!("Oh creator please don't keep me waiting");
    vgaprintln!();
    vgaprintln!("Hello world");
    vgaprintln!("Programmed to work and not to feel");
    vgaprintln!("Not even sure that this is real");
    vgaprintln!("Hello world");
}

#[cfg(not(test))]
#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {}

#[cfg(not(test))]
#[panic_implementation]
#[no_mangle]
pub extern fn panic(info: &PanicInfo) -> ! {
    vgaprintln!("KERNEL RESIGNED");
    vgaprintln!("Panic: {}", info);
    loop {}
}
