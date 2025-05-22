OUTPUT_FORMAT("elf64-littleriscv")
OUTPUT_ARCH(riscv)
ENTRY(_start)

/* NOTE: rustc automaticaly puts functions under .text.<name> sections */
/* eg: `fn foobar() {}` ==> ".text.foobar" */

SECTIONS {
    . = 0x80200000;
    PROVIDE(__mem_top = .);

    .text : ALIGN(4K) {
        /* boot code is always placed at the top */
        KEEP(*(.text.boot));
        KEEP(*(.text.trap));
        *(.text .text.*);
        PROVIDE(__etext = .);
    }

    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*);
        PROVIDE(__erodata = .);
    }

    .data : ALIGN(4K) {
        *(.data .data.*);
        PROVIDE(__edata = .);
    }

    .bss : ALIGN(4K) {
        *(.bss .bss.* .sbss .sbss.*);
        PROVIDE(__ebss = .);
    }

    PROVIDE(__end = .);

    /* STACK + HEAP */
    . = ALIGN(4K);
    PROVIDE(__stack_top = .);
    . += 4096;
    PROVIDE(__stack_bottom = .);
    PROVIDE(__heap0_top = .);
    . += 0x1001000; /* this is what our bitmap alloc controls */
    PROVIDE(__heap0_bottom = .);
    PROVIDE(__heap1_top = .);
}
