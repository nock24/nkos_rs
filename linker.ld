SECTIONS
{
    . = 0x80000;

    .text :
    {
        KEEP(*(.text.boot)) *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }

    .data :
    {
        *(.data .data.*)
    }

    .bss (NOLOAD) :
    {
        . = ALIGN(16);
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    __bss_size = __bss_end - __bss_start;

    . = ALIGN(16);
    __heap_start = .;
    __heap_size = 32M;
    __heap_end = __heap_start + __heap_size;

    __stack_top = 0x40000000;

    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
