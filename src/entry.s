# the rust compiler complains if I dont include this
.attribute arch, "rv64gc"
.option norvc

# the code in here was annoying to write in rust, this is much cleaner and easier
.section .text.boot
.global _start
_start:
    la t0, __stack_bottom
    mv sp, t0

    la t0, ktrapvec
    csrw stvec, t0

    li t0, 0x222
    csrw sie, t0

    li t0, (1 << 1)
    csrrs zero, sstatus, t0

    call start

# we also create global symbols to access the symbols we defined in linker.ld
.section .rodata

.global ETEXT
ETEXT: .dword __etext
# stack
.global STACK_TOP
STACK_TOP: .dword __stack_top
.global STACK_BOTTOM
STACK_BOTTOM: .dword __stack_bottom
# the first heap, this is for very early stage init
.global HEAP0_TOP
HEAP0_TOP: .dword __heap0_top
.global HEAP0_BOTTOM
HEAP0_BOTTOM: .dword __heap0_bottom
# the second heap, this is managed by a linked list allocator
.global HEAP1_TOP
HEAP1_TOP: .dword __heap1_top
# .global HEAP1_BOTTOM
# HEAP1_BOTTOM: .dword __heap1_bottom
