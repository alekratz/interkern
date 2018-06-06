.section .multiboot_header
header_start:

.long 0xe85250d6                   # multiboot 2 magic number
.long 0                            # architecture 0 (protected mode i386)
.long header_end - header_start    # header length
# checksum
.long -(0xe85250d6 + 0 + (header_end - header_start))

.align 8
# end tag
.word 0        # type
.word 0        # flags
.long 8        # size
header_end:
