use x86_64::{
    structures::{
        idt::{
            Idt,
            ExceptionStackFrame,
        },
        gdt::SegmentSelector,
        tss::TaskStateSegment,
    },
    instructions::{
        segmentation::set_cs,
        tables::load_tss,
    },
    VirtualAddress,
};
use spin::Once;
use memory::{MemoryController, FrameAllocator};

mod gdt;

lazy_static! {
    static ref IDT: Idt = setup_idt();
}
static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<gdt::Gdt> = Once::new();

fn setup_idt() -> Idt {
    let mut idt = Idt::new();

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    unsafe {
        idt.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
    }

    idt
}

const DOUBLE_FAULT_IST_INDEX: usize = 0;

pub fn init(memory_controller: &mut MemoryController<impl FrameAllocator>) {
    let double_fault_stack = memory_controller.alloc_stack(1)
        .expect("Could not allocate a stack for the double fault handler");
    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = VirtualAddress(double_fault_stack.top());
        tss
    });

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);
    let gdt = GDT.call_once(|| {
        let mut gdt = gdt::Gdt::new();
        code_selector = gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        tss_selector = gdt.add_entry(gdt::Descriptor::task_state_segment(&tss));
        gdt
    });
    gdt.load();

    // set the CS and load the TSS
    unsafe {
        set_cs(code_selector);
        load_tss(tss_selector);
    }

    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame) {
    vgaprintln!("BREAKPOINT EXCEPTION");
    vgaprintln!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut ExceptionStackFrame, error_code: u64) {
    vgaprintln!("= DOUBLE FAULT EXCEPTION");
    vgaprintln!("Details:");
    vgaprintln!("Error code: {:#x}", error_code);
    vgaprintln!("{:#?}", stack_frame);
    loop {}
}

/*

In the table below, if while handling any exception in the "first exception"
column, and another exception in the "second exception" column on that row
occurs, a double fault is invoked.

For example, if a general protection fault occurs while a divide-by-zero fault is being handled, a
double fault is invoked.

Otherwise, the exception is handled as normal.

First Exception             Second Exception
-------------------------+------------------
Divide-by-zero           |  Invalid TSS
Invalid TSS              |  Segment not present
Segment not present      |  Stack-segment fault
Stack-segment fault      |  General protection fault
General protection fault |
-------------------------+------------------
Page fault               |  Page fault
                         |  Invalid TSS
                         |  Segment not present
                         |  Stack-segment fault
                         |  General protection fault
*/
