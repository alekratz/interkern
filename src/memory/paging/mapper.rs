use core::ptr::Unique;
use memory::frame::{Frame, FrameAllocator};
use memory::paging::*;
pub struct Mapper {
    p4: Unique<Table<TableLevel4>>,
}

impl Mapper {
    pub unsafe fn new() -> Self {
        Mapper {
            p4: Unique::new_unchecked(P4),
        }
    }

    /// Gets a reference to the top-level page table.
    pub (in memory) fn p4(&self) -> &Table<TableLevel4> {
        unsafe { self.p4.as_ref() }
    }

    /// Gets the top-level page table mutably.
    pub (in memory) fn p4_mut(&mut self) -> &mut Table<TableLevel4> {
        unsafe { self.p4.as_mut() }
    }

    pub fn unmap<A>(&mut self, page: Page, allocator: &mut A)
        where A: FrameAllocator
    {
        // make sure that this page is actually mapped
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut()
            .next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            .expect("Hugepages are not supported yet");
        let frame = p1[page.p1_index()].to_frame().unwrap();
        p1[page.p1_index()].set_unused();
        // TODO : de-allocate above page frames if they're empty
        // TODO(arch) abstract away x86_64 calls
        use x86_64::instructions::tlb;
        use x86_64::VirtualAddress;
        tlb::flush(VirtualAddress(page.start_address()));
        // TODO(dealloc)
        //allocator.dealloc(frame);
    }

    /// Convenience function that identity maps a frame.
    pub fn identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    /// Maps a page to a yet-to-be-allocated frame.
    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let frame = allocator.alloc().expect("No available frames");
        self.map_to(page, frame, flags, allocator)
    }

    /// Maps a page to a given frame.
    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let p4 = self.p4_mut();
        let p3 = p4.next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);
        let p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(!p1[page.p1_index()].is_used(), "Attempted to use a page that is already in use: page #{:#x}", page.p1_index());
        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
    }

    /// Translate a virtual address into a (possible) physical address.
    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    /// Converts a page to a (possible) frame that it points at.
    pub (in memory) fn translate_page(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());

        // closure to help handle hugepages
        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];

                // 1GB page?
                if let Some(start_frame) = p3_entry.to_frame() {
                    if p3_entry.flags().contains(EntryFlags::HUGE) {
                        assert!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0);
                        return Some(Frame {
                            number: start_frame.number + page.p2_index() * ENTRY_COUNT + page.p1_index(),
                        });
                    }
                }
                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];
                    // 2MB page?
                    if let Some(start_frame) = p2_entry.to_frame() {
                        if p2_entry.flags().contains(EntryFlags::HUGE) {
                            assert!(start_frame.number % ENTRY_COUNT == 0);
                            return Some(Frame {
                                number: start_frame.number + page.p1_index(),
                            });
                        }
                    }
                }
                None
            })
        };

        // walk the page table to get the given frame
        p3.and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].to_frame())
            .or_else(huge_page)
    }
}
