mod frame;
mod paging;
mod heap;

pub use self::frame::*;
pub use self::paging::*;
pub use self::heap::*;

use multiboot2::{BootInformation, ElfSection};
use arch::x86_64::stack::*;

#[cfg(not(test))]
/// Initializes main memory and remaps the kernel.
pub fn init(boot_info: BootInformation) -> MemoryController<impl FrameAllocator> {
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

    let mut frame_allocator = AreaFrameAllocator::new(memory_map.memory_areas(),
        kernel_start as usize, kernel_end as usize,
        boot_info.start_address(), boot_info.end_address());

    // map the kernel and get the active page table
    let mut active_table = remap_kernel(&mut frame_allocator, &boot_info);

    // map the heap
    let heap_start = Page::containing_address(KERNEL_HEAP_START);
    let heap_end = Page::containing_address(KERNEL_HEAP_START + KERNEL_HEAP_SIZE - 1);

    vgaprintln!("Mapping heap from {:#x} to {:#x}", heap_start.start_address(), heap_end.start_address() + 4095);
    for page in Page::range_inclusive(heap_start, heap_end) {
        active_table.map(page, EntryFlags::WRITABLE, &mut frame_allocator);
    }

    // final heap initializations
    unsafe {
        let alloc_address = &::GLOBAL_ALLOCATOR as *const _ as usize;
        let alloc = &mut *(alloc_address as *mut BuddyAllocator);
        alloc.init();
    }

    // TODO(arch) pretty sure this is x86-specific
    let stack_alloc_start = heap_end + 1;
    // reserve 100 pages for interrupt stack allocation
    let stack_alloc_end = stack_alloc_start + 100;
    let stack_allocator = StackAllocator::new(Page::range_inclusive(stack_alloc_start, stack_alloc_end));

    MemoryController {
        active_table,
        frame_allocator,
        stack_allocator,
    }
}

pub struct MemoryController<F: FrameAllocator> {
    active_table: ActivePageTable,
    frame_allocator: F,
    stack_allocator: StackAllocator<PageRangeIter>,
}

impl<F: FrameAllocator> MemoryController<F> {
    pub fn alloc_stack(&mut self, size_in_pages: usize) -> Option<Stack> {
        let &mut MemoryController {
            ref mut active_table,
            ref mut frame_allocator,
            ref mut stack_allocator,
        } = self;
        stack_allocator.alloc(active_table, frame_allocator, size_in_pages)
    }
}
