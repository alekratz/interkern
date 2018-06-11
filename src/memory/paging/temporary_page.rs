use memory::{
    Page,
    EntryFlags,
    ActivePageTable,
    Frame,
    FrameAllocator,
    VirtualAddress,
    Table,
    TableLevel1,
};

/// A temporary page that exists to map inactive page tables.
pub struct TemporaryPage {
    page: Page,
    alloc: TemporaryFrameAllocator,
}

impl TemporaryPage {
    pub fn new<A>(page: Page, allocator: &mut A) -> Self
        where A: FrameAllocator
    {
        TemporaryPage {
            page,
            alloc: TemporaryFrameAllocator::new(allocator),
        }
    }

    /// Maps this temporary page to an inactive table, using the active table.
    pub fn map_to_table(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> &mut Table<TableLevel1> {
        // NOTE : we return TableLevel1 because, assumedly, the table we just created hasn't been
        // filled out yet. So we don't want to be able to call .next_table(index) on it, because
        // it'll just give back garbage pointers.
        // THIS IS ACTUALLY A P4 TABLE.
        unsafe {
            &mut *(self.map(frame, active_table) as *mut Table<TableLevel1>)
        }
    }

    pub fn map(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> VirtualAddress {
        assert!(active_table.translate_page(self.page).is_none(), "Temporary page is already mapped");
        active_table.map_to(self.page, frame, EntryFlags::WRITABLE, &mut self.alloc);
        self.page.start_address()
    }

    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page, &mut self.alloc);
    }
}

/// A frame allocator that can allocate up to three frames.
struct TemporaryFrameAllocator([Option<Frame>; 3]);

impl TemporaryFrameAllocator {
    fn new<A>(allocator: &mut A) -> Self
        where A: FrameAllocator
    {
        TemporaryFrameAllocator([
            allocator.alloc(),
            allocator.alloc(),
            allocator.alloc(),
        ])
    }
}

impl FrameAllocator for TemporaryFrameAllocator {
    fn alloc(&mut self) -> Option<Frame> {
        for frame in &mut self.0 {
            if frame.is_some() {
                return frame.take();
            }
        }
        None
    }

    fn dealloc(&mut self, frame: Frame) {
        for frame_opt in &mut self.0 {
            if frame_opt.is_none() {
                *frame_opt = Some(frame);
                return;
            }
        }
        panic!("TemporaryFrameAllocator can only hold 3 frames");
    }
}
