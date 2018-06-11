use multiboot2::{MemoryAreaIter, MemoryArea};
use memory::PhysicalAddress;

/// The default page size.
///
/// 4K pages are frequently the 
pub const PAGE_SIZE: usize = 4096;

/// A physical memory frame that has been allocated.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub (in memory) number: usize,
}

impl Frame {
    pub (in memory) fn containing_address(addr: usize) -> Self {
        Frame { number: addr / PAGE_SIZE }
    }

    pub fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    pub (in memory) fn clone(&self) -> Self {
        Frame { number: self.number }
    }

    pub (in memory) fn range_inclusive(start: Frame, end: Frame) -> impl Iterator<Item=Frame> {
        (start.number ..= end.number).map(|number| Frame { number })
    }
}

pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<Frame>;
    fn dealloc(&mut self, frame: Frame);
}

/// A simple frame allocator.
pub struct AreaFrameAllocator {
    next_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl AreaFrameAllocator {
    pub fn new(areas: MemoryAreaIter, kernel_start: usize, kernel_end: usize, multiboot_start: usize,
               multiboot_end: usize) -> Self {
        let mut alloc = AreaFrameAllocator {
            next_frame: Frame { number: 0 },
            current_area: None,
            areas,
            kernel_start: Frame::containing_address(kernel_start),
            kernel_end: Frame::containing_address(kernel_end),
            multiboot_start: Frame::containing_address(multiboot_start),
            multiboot_end: Frame::containing_address(multiboot_end),
        };
        alloc.choose_next_area();
        alloc
    }
    fn choose_next_area(&mut self) {
        self.current_area = self.areas.clone()
            .filter(|area| Frame::containing_address(area.start_address() + area.size() - 1) >= self.next_frame)
            .min_by_key(|area| area.start_address());
        if let Some(area) = self.current_area {
            let area_start_frame = Frame::containing_address(area.start_address());
            assert!(self.next_frame <= area_start_frame,
                concat!("next_frame in AreaFrameAllocator was greater than the start frame for the desired area, ",
                        "even though the area should have been filtered out"));
            if self.next_frame < area_start_frame {
                self.next_frame = area_start_frame;
            }
        }
    }
}

impl FrameAllocator for AreaFrameAllocator {
    fn alloc(&mut self) -> Option<Frame> {
        let area = self.current_area?;
        let frame = Frame { number: self.next_frame.number };

        // last frame of the current memory area
        let area_last_frame = Frame::containing_address(area.start_address() + area.size() - 1);

        // last frame of the current area is too small for our next frame, so advance
        if frame > area_last_frame {
            self.choose_next_area();
        } else if frame >= self.kernel_start && frame <= self.kernel_end {
            self.next_frame = Frame { number: frame.number + 1 };
        } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
            self.next_frame.number += 1;
        } else {
            // if all the checks passed, then this frame is free and we can allocate it
            self.next_frame.number += 1;
            return Some(frame);
        }
        // since we're messing with the "next frame" and "current memory area", maybe the next
        // allocation will succeed
        self.alloc()
    }

    fn dealloc(&mut self, frame: Frame) {
        unimplemented!()
    }
}
