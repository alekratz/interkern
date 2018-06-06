ENTRY(_start)

SECTIONS {
    . = 1M;

    .text ALIGN(4K) :
    {
        KEEP(*(.multiboot_header))
        *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }

    .data.rel.ro :
    {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    }

    .bss :
    {
        *(.bss)
    }
}
