.section .rodata

.global MEMTOP
MEMTOP: .dword __mem_top

.global ETEXT
ETEXT: .dword __etext

# our rust code does not have any need to access these as of yet
# .global STACK_TOP
# STACK_TOP: .dword __stack_top
# .global STACK_BOTTOM
# STACK_BOTTOM: .dword __stack_bottom

# the first heap, this is for very early stage init
.global HEAP0_TOP
HEAP0_TOP: .dword __heap0_top

# the second heap, this is managed by a linked list allocator
.global HEAP1_TOP
HEAP1_TOP: .dword __heap1_top
