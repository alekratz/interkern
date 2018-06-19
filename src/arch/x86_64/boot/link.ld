ENTRY(_start)

KERNEL_START = 0xffff800000000000;

SECTIONS {
    . = 1M;

    .rodata.early ALIGN(4K) :
    {
        KEEP(*(.multiboot_header))
        *(.rodata.early)
    }

    .text.early ALIGN(4K) :
    {
        *(.text.early)
    }

    .bss.early ALIGN(4K) :
    {
        *(.bss.early)
    }

    __early_end = .;

    . += KERNEL_START;

    .text ALIGN(4K) : AT(ADDR(.text) - KERNEL_START)
    {
        *(.text .text.*)
    }

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - KERNEL_START)
    {
        *(.rodata .rodata.*)
    }

    .data.rel.ro ALIGN(4K) : AT(ADDR(.data.rel.ro) - KERNEL_START)
    {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    }

    .data ALIGN(4K) : AT(ADDR(.data) - KERNEL_START)
    {
        *(.data .data.*)
    }

    .bss ALIGN(4K) : AT(ADDR(.bss) - KERNEL_START)
    {
        *(.bss .bss.*)
    }

    .got ALIGN(4K) : AT(ADDR(.got) - KERNEL_START)
    {
        *(.got)
    }

    .got.plt ALIGN(4K) : AT(ADDR(.got.plt) - KERNEL_START)
    {
        *(.got.plt)
    }

    .gcc_except_table ALIGN(4K) : AT(ADDR(.gcc_except_table) - KERNEL_START)
    {
        *(.gcc_except_table)
    }
}
