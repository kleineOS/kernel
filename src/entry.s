# the code in here was annoying to write in rust, this is much cleaner and easier
.section .text.boot
.global _start
_start:
    # load the first 32 bits from the addr stored in a1
    lwu t0, 0(a1)
    li t1, 0xedfe0dd0 # the magic value for fdt parsing

    bne t0, t1, alloced_stack
    j static_stack

static_stack:
    la sp, __stack_bottom
    j setstvec
alloced_stack:
    mv sp, a1
    j setstvec

setstvec:
    la t2, ktrapvec
    csrw stvec, t2
setsie:
    li t2, 0x222
    csrw sie, t2
setsstatus:
    li t2, (1 << 1)
    csrrs zero, sstatus, t2
callstart:
    bne t0, t1, bootstrap_core
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
