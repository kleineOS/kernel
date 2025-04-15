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
# we start with the symbols for our stack. __stack_bottom is to be moved into
# the stack pointer, as a stack grows from high to low
.global STACK_TOP
STACK_TOP: .dword __stack_top
.global STACK_BOTTOM
STACK_BOTTOM: .dword __stack_bottom
# next is our heap. this is allocated right after the bottom of our stack. the
# heap grows from low to high
.global HEAP_TOP
HEAP_TOP: .dword __heap_top
