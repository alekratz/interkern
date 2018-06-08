use memory::{PAGE_SIZE, Frame, FrameAllocator};

mod table;

pub use self::table::*;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

/// A virtual page of memory that maps to a physical frame.
#[derive(Debug, Clone, Copy)]
pub struct Page {
    number: usize,
}

impl Page {
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

/// An entry in a page table.
///
/// Page table entries for x86_64 are 64 bits wide.
///
/// TODO(arch) this is x86_64 specific
pub struct Entry(u64);

impl Entry {
    pub fn is_used(&self) -> bool {
        self.0 != 0
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    /// Gets the flags of this entry.
    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    /// Gets the physical frame that this virtual page entry is pointing at, if it is present.
    pub fn to_frame(&self) -> Option<Frame> {
        if self.flags().contains(EntryFlags::PRESENT) {
            let addr = self.0 as usize & 0x000fffff_fffff000;
            Some(Frame::containing_address(addr))
        } else {
            None
        }
    }

    /// Sets the frame and flags for this entry.
    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        assert!(frame.start_address() & !0x000fffff_fffff000 == 0, "Physical frame address is not page-aligned");
        self.0 = (frame.start_address() as u64) | flags.bits();
    }
}

/// Flags for page table entries.
bitflags! {
    pub struct EntryFlags: u64 {
        const PRESENT       = 1 << 0;
        const WRITABLE      = 1 << 1;
        const USER          = 1 << 2;
        const WRITETHROUGH  = 1 << 3;
        const DISABLECACHE  = 1 << 4;
        const ACCESSED      = 1 << 5;
        const DIRTY         = 1 << 6;
        const HUGE          = 1 << 7;
        const GLOBAL        = 1 << 8;
        // bits 9-11 and 52-62 are unused by the CPU
        const NOEXEC        = 1 << 63;
    }
}

/*
pub fn test_paging<A>(allocator: &mut A)
    where A: FrameAllocator
{
    let mut page_table = unsafe { ActivePageTable::new() };

    let addr = 42 * 512 * 512 * 4096; // 42nd P3 entry
    let page = Page::containing_address(addr);
    let frame = allocator.alloc().expect("No free frames");
    vgaprintln!("None = {:?}, map to {:?}", page_table.translate(addr), frame);
    page_table.map_to(page, frame, EntryFlags::empty(), allocator);
    vgaprintln!("Some = {:?}", page_table.translate(addr));
    vgaprintln!("Virtual address of frame: {:x}", addr);
    vgaprintln!("Next free frame: {:?}", allocator.alloc());
    vgaprintln!("Virtual address: {:#x}", addr);
    vgaprintln!("{:#x}", unsafe {
        *(Page::containing_address(addr).start_address() as *const u64)
    });
    vgaprintln!("Unmapping page");
    page_table.unmap(Page::containing_address(addr), allocator);

    vgaprintln!("Invoking page fault");
    vgaprintln!("{:#x}", unsafe {
        *(Page::containing_address(addr).start_address() as *const u64)
    });
}
*/
