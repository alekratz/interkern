//! The operating system's memory map.
//!
//! Every place in memory must be mapped in such a way that it does not overlap. This module helps
//! lay out all memory regions in a single location so that conflicts are easy to spot.

/// Base for user-space virtual addresses.
pub const USER_BASE: usize                              = 0x0000_0000_0010_0000
        ;

/// The start address for the kernel.
///
/// This starts 3/4s the way up in virtual memory.
pub const KERNEL_BASE: usize                            = 0xffff_8000_0000_0000
        ;
    // NOTE : This should match what's in arch/x86_64/boot/link.ld
// 0o177_777_400_000_000_000_0000


/// This is an exported symbol that `ld` is able to use.

/// The start address for the kernel's heap.
pub const KERNEL_HEAP_START: usize                      = 0x0000_0000_4000_0000
        + KERNEL_BASE;

