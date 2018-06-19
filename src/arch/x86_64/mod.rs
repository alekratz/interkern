#[macro_use] pub mod vga;
pub mod interrupt;
pub mod stack;
pub mod cpu;

/// Enables various features on the EFER register.
///
/// The features which are enabled are:
/// * the NXE bit, which allows setting pages to be non-executable
/// * the SYSCALL enable bit, which allows usage of the `syscall` instruction
///
/// This allows us to set pages as NO_EXECUTE.
pub fn enable_efer_features() {
    use x86_64::registers::msr::{IA32_EFER, rdmsr, wrmsr};

    let syscall_bit = 1 << 0;
    let nxe_bit     = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit | syscall_bit);
    }
}

/// Enables write protection of pages in kernel mode.
///
/// By default, when in kernel mode, x86 ignores the write-protect bit on pages. This disables this
/// functionality.
pub fn enable_kernel_write_protect() {
    use x86_64::registers::control_regs::{cr0, cr0_write, Cr0};

    unsafe {
        cr0_write(cr0() | Cr0::WRITE_PROTECT);
    }
}
