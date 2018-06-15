use memory::{
    Page, ActivePageTable, PAGE_SIZE, EntryFlags,
    FrameAllocator,
};

pub struct Stack {
    top: usize,
    bottom: usize,
}

impl Stack {
    fn new(top: usize, bottom: usize) -> Self {
        Stack { top, bottom }
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn bottom(&self) -> usize {
        self.bottom
    }
}

pub struct StackAllocator<I: Iterator<Item=Page> + Clone> {
    range: I,
}

impl<I: Iterator<Item=Page> + Clone> StackAllocator<I> {
    pub fn new(range: I) -> Self {
        StackAllocator { range, }
    }

    pub fn alloc<A: FrameAllocator>(&mut self, active_table: &mut ActivePageTable, allocator: &mut A,
                                    size_in_pages: usize) -> Option<Stack> {
        if size_in_pages == 0 {
            return None;
        }

        let mut range = self.range.clone();
        let guard_page = range.next();
        let stack_start = range.next();

        let stack_end = if size_in_pages == 1 {
            stack_start
        } else {
            range.nth(size_in_pages - 2)
        };

        match (guard_page, stack_start, stack_end) {
            (Some(_), Some(start), Some(end)) => {
                self.range = range;
                for page in Page::range_inclusive(start, end) {
                    active_table.map(page, EntryFlags::WRITABLE, allocator);
                }
                let stack_top = end.start_address() + PAGE_SIZE;
                Some(Stack::new(stack_top, start.start_address()))
            },
            _ => None,
        }
    }
}
