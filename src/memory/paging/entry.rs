use multiboot2::{
    ElfSection,
    ElfSectionFlags,
};
use memory::Frame;

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

impl From<ElfSectionFlags> for EntryFlags {
    fn from(section: ElfSectionFlags) -> Self {
        let mut flags = EntryFlags::empty();

        if section.contains(ElfSectionFlags::ALLOCATED) {
            flags |= EntryFlags::PRESENT;
        }
        if section.contains(ElfSectionFlags::WRITABLE) {
            flags |= EntryFlags::WRITABLE;
        }
        if !section.contains(ElfSectionFlags::EXECUTABLE) {
            flags |= EntryFlags::NOEXEC;
        }

        flags
    }
}