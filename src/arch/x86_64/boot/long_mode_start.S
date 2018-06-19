.code64
.section .text.early
.global _long_mode_start

.extern kmain

_long_mode_start:
    # Fix up those segments
    mov $0, %ax
    mov %ax, %ss
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    #call kmain
    movabs $kmain, %rax
    jmp *%rax
    hlt
