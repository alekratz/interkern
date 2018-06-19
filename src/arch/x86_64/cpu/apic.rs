use memory::{ActivePageTable, Page};

/// Initializes the APIC chip on the Intel CPU.
pub fn init(active_table: &mut ActivePageTable) {
    // map the APIC to wherever we want it to live
}
