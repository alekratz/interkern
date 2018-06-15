mod frame;
mod paging;
mod heap;

pub use self::frame::*;
pub use self::paging::*;
pub use self::heap::*;

use multiboot2::{BootInformation, ElfSection};

#[cfg(not(test))]
/// Initializes main memory and remaps the kernel.
pub fn init(boot_info: BootInformation) {
    //assert_has_not_been_called!("memory::init must be called exactly once");

    let memory_map = boot_info.memory_map_tag()
        .expect("Could not find memory map tag in multiboot2 data");
    let elf_sections = boot_info.elf_sections_tag()
        .expect("Could not find ELF sections tag in multiboot2 data");

    let kernel_start = elf_sections.sections()
        .filter(ElfSection::is_allocated)
        .map(|s| s.start_address())
        .min()
        .unwrap();
    let kernel_end = elf_sections.sections()
        .filter(ElfSection::is_allocated)
        .map(|s| s.start_address() + s.size())
        .max()
        .unwrap();

    let mut allocator = AreaFrameAllocator::new(memory_map.memory_areas(),
        kernel_start as usize, kernel_end as usize,
        boot_info.start_address(), boot_info.end_address());

    // map the kernel and get the active page table
    let mut active_table = remap_kernel(&mut allocator, &boot_info);

    // map the heap
    let heap_start = Page::containing_address(KERNEL_HEAP_START);
    let heap_end = Page::containing_address(KERNEL_HEAP_START + KERNEL_HEAP_SIZE - 1);

    vgaprintln!("Mapping heap from {:#x} to {:#x}", heap_start.start_address(), heap_end.start_address() + 4095);
    for page in Page::range_inclusive(heap_start, heap_end) {
        active_table.map(page, EntryFlags::WRITABLE, &mut allocator);
    }

    // final heap initializations
    unsafe {
        let alloc_address = &::GLOBAL_ALLOCATOR as *const _ as usize;
        let alloc = &mut *(alloc_address as *mut BuddyAllocator);
        alloc.init();
    }
}
