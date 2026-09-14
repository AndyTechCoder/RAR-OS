/* Fixed native app virtual layout; no host ABI or dynamic runtime. */
OUTPUT_FORMAT(elf64-x86-64)
ENTRY(efi_main)
PHDRS
{
  text PT_LOAD FLAGS(5);
  rodata PT_LOAD FLAGS(4);
  data PT_LOAD FLAGS(6);
}
SECTIONS
{
  . = 0x401000;
  .text : { *(.text .text.*) } :text
  . = ALIGN(4096);
  .rodata : { *(.rodata .rodata.*) } :rodata
  . = ALIGN(4096);
  .data : { *(.data .data.*) *(.bss .bss.* COMMON) } :data
  /DISCARD/ : { *(.comment .note.GNU-stack) }
}
