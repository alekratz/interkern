.code32

# GDT for 64-bit transition
.section .rodata.early
gdt64:
    .quad 0
.set GDT_CODESEG, . - gdt64
    # Code segment
    .quad (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53)
gdt64_pointer:
    .word . - gdt64 - 1
    .quad gdt64

.section .text.early

.extern _long_mode_start

# Program entry point
.global _start
_start:
    leal stack_top, %esp
    # Multiboot info pointer gets stored in %edi
    movl %ebx, %edi
    # Startup CPU checks
    call _check_multiboot
    call _check_cpuid
    call _check_long_mode

    # Init and start paging
    call _init_page_tables
    call _enable_paging
    lgdt gdt64_pointer
    jmp $GDT_CODESEG, $_long_mode_start

# Multiboot check
_check_multiboot:
    cmp $0x36d76289, %eax
    jne 1f
    ret
1:
    movb $'0, %al
    jmp _error

# CPUID check
_check_cpuid:
    # Attempt to fit bit 21 in the FLAGS register
    # If we can fip it, the CPUID is available.
    pushf
    popl %eax
    movl %eax, %ecx
    xorl $1 << 21, %eax
    pushl %eax
    popf
    pushf
    popl %eax
    # Restore the old FLAGS
    pushl %ecx
    popf
    cmp %eax,%ecx
    je 1f
    ret
1:
    movb $'1, %al
    jmp _error

# Check if this CPU supports moving to long mode
_check_long_mode:    
    # Check if extended processor info is available
    movl $0x80000000, %eax
    cpuid
    cmpl $0x80000001, %eax
    # If not, long mode is definitely not available
    jb 1f
    # Check if long mode is available
    movl $0x80000001, %eax
    cpuid
    # Bit 29 is the "long mode" bit and gets put into EDX
    # If it isn't set, long mode is not supported
    test $1 << 29, %edx
    jz 1f
    ret
1:
    movb $'2, %al
    jmp _error

# Initialize page tables
_init_page_tables:
    # Map the last entry of P4 to itself for recursive table mapping
    leal p4_table, %eax
    orl $0b011, %eax  # present, writable
    movl $511, %esi
    movl %eax, p4_table(,%esi,8)
    # Map entry 0x100 of P4 to the P3 table
    leal p3_table, %eax
    # Present, writeable
    orl $0b11,%eax
    movl $0x100, %esi
    movl %eax, p4_table(, %esi, 8)
    # Also map entry 0
    movl %eax, p4_table

    # Map first entry of P3 to the P2 table
    leal p2_table, %eax
    orl $0b11,%eax
    movl %eax, p3_table

    # Map all entries of P2 to be hugepages
    movl $0, %ecx
1:
    movl $0x200000, %eax
    mull %ecx
    # hugepage + write + present
    orl $0b000010000011, %eax
    movl %eax, p2_table(, %ecx, 8)
    inc %ecx
    cmp $512, %ecx
    jne 1b
    ret

# Enable paging in the processor
_enable_paging:
    # Move P4 table address to CR3 register
    leal p4_table, %eax
    movl %eax, %cr3
    # Enable PAE
    movl %cr4, %eax
    orl $1 << 5, %eax
    movl %eax, %cr4
    # Set long mode bit in the EFER MSR
    movl $0xc0000080, %ecx
    rdmsr
    orl $1 << 8, %eax
    wrmsr
    # Enable paging with the CR0 register
    movl %cr0, %eax
    orl $1 << 31, %eax
    movl %eax, %cr0
    # Done!
    ret

# Rudimentary error code operation
_error:
    movl $0x4f524f45, (0xb8000)
    movl $0x4f3a4f52, (0xb8004)
    movl $0x4f204f20, (0xb8008)
    movb %al, (0xb800a)
    hlt

.section .bss.early
# Page table
.align 4096
.global p4_table
p4_table: .fill 4096
p3_table: .fill 4096
p2_table: .fill 4096

# Stack
stack_bottom:
.fill 4096 * 4
stack_top:

