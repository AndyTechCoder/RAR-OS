OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

SECTIONS
{
    . = 0x00100000;
    .text : ALIGN(4096) { KEEP(*(.text.entry)) *(.text*) }
    .rodata : ALIGN(4096) { *(.rodata*) }
    /DISCARD/ : { *(.comment) *(.eh_frame*) *(.note*) }
}
