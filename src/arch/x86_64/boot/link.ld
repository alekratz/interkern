ENTRY(_start)

SECTIONS {
    . = 1M;

    .rodata ALIGN(4K) :
    {
        KEEP(*(.multiboot_header))
        *(.rodata .rodata.*)
    }

    .text ALIGN(4K) :
    {
        *(.text .text.*)
    }

    .data.rel.ro ALIGN(4K) :
    {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    }

    .data ALIGN(4K) :
    {
        *(.data .data.*)
    }

    .bss ALIGN(4K) :
    {
        *(.bss .bss.*)
    }

    .got ALIGN(4K) :
    {
        *(.got)
    }

    .got.plt ALIGN(4K) :
    {
        *(.got)
    }

    .gcc_except_table ALIGN(4K) :
    {
        *(.gcc_except_table)
    }
}
