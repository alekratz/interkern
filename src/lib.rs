#![feature(lang_items, panic_implementation, const_fn, ptr_internals)]
#![no_std]

extern crate volatile;
extern crate spin;
extern crate multiboot2;
#[macro_use] extern crate bitflags;
// TODO(arch) this should be conditional on the project configuration
extern crate x86_64;

#[macro_use] pub mod arch;
pub mod memory;

use core::panic::PanicInfo;
use memory::FrameAllocator;

extern {
    static p4_table: usize;
}

/// The kernel entrypoint.
///
/// # Arguments
/// `mb2_info_addr` - the address where multiboot2 system information resides.
#[no_mangle]
pub extern fn kmain(mb2_info_addr: usize) {
    // TODO(arch) this is x86_64 specific
    arch::x86_64::enable_nxe_bit();
    arch::x86_64::enable_kernel_write_protect();

    welcome();

    vgaprintln!();
    vgaprintln!("{}", "================================================================================");
    vgaprintln!();

    let mb2_info = unsafe { multiboot2::load(mb2_info_addr) };
    let memory_map = mb2_info.memory_map_tag()
        .expect("Could not find memory map tag in multiboot2 data");
    vgaprintln!("usable memory areas:");
    for area in memory_map.memory_areas() {
        vgaprintln!("   start: 0x{:x}, length 0x{:x}", area.start_address(), area.size());
    }

    let elf_sections = mb2_info.elf_sections_tag()
        .expect("Could not find ELF sections tag in multiboot2 data");
    vgaprintln!("ELF sections:");
    for section in elf_sections.sections() {
        vgaprintln!("    addr: {:#x}, size: {:#x}, flags: {:#b}", section.start_address(), section.size(), section.flags());
    }

    let kernel_start = elf_sections.sections()
        .map(|s| s.start_address())
        .min()
        .unwrap();
    let kernel_end = elf_sections.sections()
        .map(|s| s.start_address() + s.size())
        .max()
        .unwrap();
    //vgaprintln!("Kernel start address: 0x{:x}", kernel_start);
    //vgaprintln!("Kernel end address: 0x{:x}", kernel_end);
    vgaprintln!("Kernel size in memory: 0x{:x} bytes", kernel_end - kernel_start);

    let mut allocator = memory::AreaFrameAllocator::new(memory_map.memory_areas(),
        kernel_start as usize, kernel_end as usize, mb2_info.start_address(), mb2_info.end_address());
    memory::remap_kernel(&mut allocator, &mb2_info);
    //let frame = allocator.alloc();
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

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {}

#[panic_implementation]
#[no_mangle]
pub extern fn panic(info: &PanicInfo) -> ! {
    vgaprintln!("KERNEL RESIGNED");
    vgaprintln!("Panic: {}", info);
    loop {}
}

