/*
 * Non-executing R0-003 certification candidate.
 *
 * Entry ABI: RDI = immutable BootEntryV1 physical address, RSI = byte length.
 * Preconditions and invariants are specified in docs/release-0/x86_64/preauth.md.
 * This file is assembled and inspected only during Prompt 7A. It must not execute.
 */

.section .text.entry,"ax",@progbits
.global _start
.type _start,@function
_start:
    cli
    cld
    test %rdi, %rdi
    jz .Linvalid_entry
    cmp $64, %rsi
    jb .Ltruncated
    cmp $4096, %rsi
    ja .Loversized
    test $7, %rdi
    jnz .Lbad_alignment

    /* Prompt 7A stops before entry-derived acquisition or target execution. */
    mov $0, %eax
    jmp .Lsealed_halt

.Ltruncated:
    mov $1, %eax
    jmp .Lsealed_halt
.Loversized:
    mov $2, %eax
    jmp .Lsealed_halt
.Lbad_alignment:
    mov $9, %eax
    jmp .Lsealed_halt
.Linvalid_entry:
    mov $30, %eax
.Lsealed_halt:
    cli
    hlt
    jmp .Lsealed_halt
.size _start, .-_start

.section .note.GNU-stack,"",@progbits
