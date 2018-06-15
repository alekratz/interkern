use core::mem::size_of;
use x86_64::{
    instructions::tables::{
        DescriptorTablePointer,
        lgdt
    },
    structures::{
        tss::TaskStateSegment,
        gdt::SegmentSelector,
    },
    PrivilegeLevel,
};
use bit_field::BitField;

pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

impl Descriptor {
    pub fn kernel_code_segment() -> Self {
        let flags = DescriptorFlags::USER_SEGMENT | DescriptorFlags::PRESENT
                  | DescriptorFlags::EXECUTABLE | DescriptorFlags::LONG_MODE;
        Descriptor::UserSegment(flags.bits())
    }

    pub fn task_state_segment(tss: &'static TaskStateSegment) -> Self {
        //use DescriptorFlags::*;
        let ptr = tss as *const _ as u64;

        let mut low = DescriptorFlags::PRESENT.bits();
        // base tss address
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        // -1 because the bound is inclusive(????? seriously guys?)
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
        // 0b1001 = present, 64-bit tss
        low.set_bits(40..44, 0b1001);

        let mut high = 0;
        // the last 32 bits of the TSS address goes into here
        high.set_bits(0..32, ptr.get_bits(32..64));

        Descriptor::SystemSegment(low, high)
    }
}

bitflags! {
    struct DescriptorFlags: u64 {
        const CONFORMING   = 1 << 42;
        const EXECUTABLE   = 1 << 43;
        const USER_SEGMENT = 1 << 44;
        const PRESENT      = 1 << 47;
        const LONG_MODE    = 1 << 53;
    }
}

pub struct Gdt {
    table: [u64; 8],
    next_free: usize,
}

impl Gdt {
    pub fn new() -> Self {
        Gdt {
            table: [0; 8],
            next_free: 1,
        }
    }

    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(flags) => self.push(flags),
            Descriptor::SystemSegment(low, high) => {
                let index = self.push(low);
                self.push(high);
                index
            },
        };
        SegmentSelector::new(index as u16, PrivilegeLevel::Ring0)
    }

    fn push(&mut self, entry: u64) -> usize {
        if self.next_free < self.table.len() {
            let index = self.next_free;
            self.next_free += 1;
            self.table[index] = entry;
            index
        } else {
            panic!("GDT table overflow");
        }
    }

    pub fn load(&'static self) {
        let ptr = DescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            limit: (self.table.len() * size_of::<u64>() - 1) as u16,
        };
        unsafe { lgdt(&ptr) };
    }
}
