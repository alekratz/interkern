use multiboot2::BootInformation;
use memory::{PAGE_SIZE, Frame, FrameAllocator};

mod entry;
mod table;
mod temporary_page;
mod mapper;

pub use self::entry::*;
pub use self::table::*;
pub use self::temporary_page::*;
pub use self::mapper::Mapper;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

/// A virtual page of memory that maps to a physical frame.
#[derive(Debug, Clone, Copy)]
pub struct Page {
    number: usize,
}

impl Page {
    pub fn range_inclusive(start: Page, end: Page) -> impl Iterator<Item=Page> {
        (start.number ..= end.number).map(|number| Page { number })
    }

    pub fn containing_address(address: VirtualAddress) -> Self {
        assert!(address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000,
                "invalid address passed to Page::containing_address: 0x{:x}", address);
        Page { number: address / PAGE_SIZE }
    }

    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    pub fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    pub fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    pub fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    pub fn p1_index(&self) -> usize {
        self.number & 0o777
    }
}

pub fn remap_kernel<A>(allocator: &mut A, boot_info: &BootInformation) -> ActivePageTable
    where A: FrameAllocator
{
    let mut temporary_page = TemporaryPage::new(Page { number: 0xDECAFDAD }, allocator);
    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.alloc().expect("No frames available");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        let elf_sections_tag = boot_info.elf_sections_tag()
            .expect("ELF sections tag memory map not available");
        for section in elf_sections_tag.sections() {
            if !section.is_allocated() {
                continue; // don't bother allocating unused sections
            }
            assert!(section.start_address() % PAGE_SIZE as u64 == 0,
                    "ELF Sections must be page-aligned (got {:#x} instead)", section.start_address());
            let flags = EntryFlags::from(section.flags().clone());
            let start_frame = Frame::containing_address(section.start_address() as usize);
            let end_frame = Frame::containing_address(section.end_address() as usize - 1);
            vgaprintln!("Identity mapping {:#x} - {:#x} flags {:#x}", start_frame.start_address(), end_frame.start_address() + 4095, flags);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }
        }
        let mb_start = Frame::containing_address(boot_info.start_address() as usize);
        let mb_end = Frame::containing_address(boot_info.end_address() as usize - 1);
        for frame in Frame::range_inclusive(mb_start, mb_end) {
            mapper.identity_map(frame, EntryFlags::PRESENT, allocator);
        }
        let vga_buffer_frame = Frame::containing_address(0xb8000);
        mapper.identity_map(vga_buffer_frame, EntryFlags::WRITABLE, allocator);
        
    });
    let old_table = active_table.switch(new_table);

    // Reuse the old p2 and p3 tables for stack, with the old p4 table becoming a stack guard
    // we can use the frame address because it's identity mapped
    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page, allocator);
    vgaprintln!("Stack guard page at {:#x}", old_p4_page.start_address());

    active_table
}
