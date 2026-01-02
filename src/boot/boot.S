.section ".text.boot"

.global _start

_start:
    /* Halt secondary cores */
    mrs     x0, mpidr_el1
    and     x0, x0, #3
    cbz     x0, 2f
1:
    wfe
    b       1b

2:
    ldr     x0, =__stack_top
    mov     sp, x0

    /* Clear BSS */
    ldr     x0, =__bss_start
    ldr     x1, =__bss_end
3:
    cmp     x0, x1
    b.hs    4f
    str     xzr, [x0], #8
    b       3b

4:
    /* Jump to kernel main */
    bl      kernel_main

5:
    /* Failsafe halt */
    wfe
    b       5b

