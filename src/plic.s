# unused code from when I was experimenting with the PLIC
# right now, I need to focus on Allocators and a common API to make drivers
# API needs to be C-compatible while also having proper rust support, because I
# do want my friend (who likes Vlang) to be able to use it for graphics programming

plic:
    li t0, 0x0C000000
    li t1, 10
    li t2, 7
    slli t1, t1, 2
    add t1, t0, t1
    sw t2, 0(t1)

    li t0, 0x0C002080
    li t1, 10
    li t2, 1
    sll t2, t2, t1
    lw t3, 0(t0)
    or t3, t3, t2
    sw t3, 0(t0)

    li t0, 0x0C201000
    li t1, 0
    sw t1, 0(t0)
    ret

uart:
    li t0, 0x10000000
    
    li t1, 0x03
    li t2, 0x80
    sb t2, 3(t0)
    sb t1, 0(t0)
    sb zero, 1(t0)
    
    li t1, 0x03
    sb t1, 3(t0)
    
    li t1, 0x01
    sb t1, 2(t0)
    
    li t1, 0x01
    sb t1, 1(t0)
    ret
