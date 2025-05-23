.section .text.boot
.global _start
_start:
    # load the first 32 bits from the addr stored in a1
    lwu t0, 0(a1)
    # the magic value for fdt parsing, be encoded
    li t1, 0xedfe0dd0

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
    bne t0, t1, kinit
    call start
