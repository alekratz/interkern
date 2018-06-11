use core::ops::{Index, IndexMut, Deref, DerefMut};
use core::marker::PhantomData;
use memory::frame::{Frame, FrameAllocator};
use memory::paging::{
    Entry, EntryFlags, Mapper,
    temporary_page::TemporaryPage,
};

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
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Self::Target {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapper
    }
}

impl ActivePageTable {
    pub (in memory) unsafe fn new() -> Self {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn with<F>(&mut self, table: &mut InactivePageTable, temporary_page: &mut TemporaryPage, f: F)
        where F: FnOnce(&mut Mapper)
    {
        use x86_64::instructions::tlb;
        use x86_64::registers::control_regs;

        {
            let old_p4 = Frame::containing_address( control_regs::cr3().0 as usize );

            let p4_table = temporary_page.map_to_table(old_p4.clone(), self);

            // overwrite the recursive mapping with the inactive page table
            self.p4_mut()[511].set(table.p4_frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb::flush_all();

            // call whatever function was passed
            f(self);

            // restore recursive mapping to the previous p4 table
            p4_table[511].set(old_p4, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb::flush_all();
        }

        temporary_page.unmap(self);
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        use x86_64::PhysicalAddress;
        use x86_64::registers::control_regs;;

        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(control_regs::cr3().0 as usize),
        };
        unsafe {
            control_regs::cr3_write(PhysicalAddress(new_table.p4_frame.start_address() as u64));
        }
        old_table
    }
}

/// An page table that is not currently active.
pub struct InactivePageTable {
    pub(in memory) p4_frame: Frame,
}

impl InactivePageTable {
    /// Creates a new inactive page table.
    ///
    /// # Arguments
    /// `p4_frame` - the allocated frame to use for this new page table.
    /// `active_table` - the active page table used to access the inactive page table.
    /// `temporary_page` - the temporary page that is used to store the inactive page table while
    ///                    we write to it.
    pub fn new(p4_frame: Frame, active_table: &mut ActivePageTable, temporary_page: &mut TemporaryPage) -> Self {
        {
            // create the new table from the temporary page
            let table = temporary_page.map_to_table(p4_frame.clone(), active_table);
            table.zero();

            // recursively map this table's frame
            table[511].set(p4_frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        }
        temporary_page.unmap(active_table);

        InactivePageTable { p4_frame, }
    }
}

/// Level 4 (top) page table.
pub const P4: *mut Table<TableLevel4> = 0o177777_777_777_777_777_0000 as *mut _;
