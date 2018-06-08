use core::ops::{Index, IndexMut};
use core::marker::PhantomData;
use core::ptr::Unique;
use memory::frame::{Frame, FrameAllocator};
use memory::paging::{Page, Entry, EntryFlags, VirtualAddress, PhysicalAddress, PAGE_SIZE};

/// The number of page entries per page table.
pub (in memory) const ENTRY_COUNT: usize = 512;

pub trait TableLevel {}

pub enum TableLevel4 {}
pub enum TableLevel3 {}
pub enum TableLevel2 {}
pub enum TableLevel1 {}

impl TableLevel for TableLevel4 {}
impl TableLevel for TableLevel3 {}
impl TableLevel for TableLevel2 {}
impl TableLevel for TableLevel1 {}

pub trait TableLevelHeirarchy: TableLevel {
    type NextLevel: TableLevel;
}

impl TableLevelHeirarchy for TableLevel4 {
    type NextLevel = TableLevel3;
}

impl TableLevelHeirarchy for TableLevel3 {
    type NextLevel = TableLevel2;
}

impl TableLevelHeirarchy for TableLevel2 {
    type NextLevel = TableLevel1;
}

/// A page table.
pub struct Table<L>
    where L: TableLevel
{
    entries: [Entry; ENTRY_COUNT],
    _level: PhantomData<L>,
}

impl<L> Table<L>
    where L: TableLevel
{
    /// Zeroes out this page table.
    pub fn zero(&mut self) {
        self.entries.iter_mut()
            .for_each(Entry::set_unused);
    }
}

impl<L> Table<L>
    where L: TableLevelHeirarchy
{
    /// Gets the address of the next page table down the line.
    ///
    /// If we are at the last page table, or if the next page table at the given index doesn't
    /// exist, `None` is returned.
    fn next_table_address(&self, index: usize) -> Option<usize> {
        let flags = self.entries[index].flags();
        if flags.contains(EntryFlags::PRESENT) && !flags.contains(EntryFlags::HUGE) {
            let table_address = self as *const _ as usize;
            Some((table_address << 9) | (index << 12))
        } else {
            None
        }
    }

    /// Gets a reference to the next page table down the line.
    ///
    /// If we are at the last page table, or if the next page table at the given index doesn't
    /// exist, `None` is returned.
    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|addr| unsafe { &*(addr as *const _)})
    }

    /// Gets a mutable reference to the next page table down the line.
    ///
    /// If we are at the last page table, or if the next page table at the given index doesn't
    /// exist, `None` is returned.
    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|addr| unsafe { &mut *(addr as *mut _)})
    }

    pub fn next_table_create<A>(&mut self, index: usize, alloc: &mut A) -> &mut Table<L::NextLevel>
        where A: FrameAllocator
    {
        if self.next_table(index).is_none() {
            assert!(!self.entries[index].flags().contains(EntryFlags::HUGE), "Hugepages are not allowed yet");
            let frame = alloc.alloc().expect("No available frames");
            self.entries[index].set(frame, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            self.next_table_mut(index).unwrap().zero();
        }
        self.next_table_mut(index).unwrap()
    }
}

impl<L> Index<usize> for Table<L>
    where L: TableLevel
{
    type Output = Entry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L>
    where L: TableLevel
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

pub struct ActivePageTable {
    p4: Unique<Table<TableLevel4>>,
}

impl ActivePageTable {
    pub unsafe fn new() -> Self {
        ActivePageTable {
            p4: Unique::new_unchecked(P4),
        }
    }

    /// Gets a reference to the top-level page table.
    fn p4(&self) -> &Table<TableLevel4> {
        unsafe { self.p4.as_ref() }
    }

    /// Gets the top-level page table mutably.
    fn p4_mut(&mut self) -> &mut Table<TableLevel4> {
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

        assert!(!p1[page.p1_index()].is_used(), "Attempted to use a page that is already in use");
        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
    }

    /// Translate a virtual address into a (possible) physical address.
    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    /// Converts a page to a (possible) frame that it points at.
    fn translate_page(&self, page: Page) -> Option<Frame> {
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

/// Level 4 (top) page table.
pub const P4: *mut Table<TableLevel4> = 0o177777_777_777_777_777_0000 as *mut _;
