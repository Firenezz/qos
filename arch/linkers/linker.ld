 /* Tell the linker that we want a riscv64 ELF64 output file */
OUTPUT_FORMAT(elf64-littleriscv)

ENTRY(boot_main)

/* Define the program headers we want so the bootloader gives us the right */
/* MMU permissions */
PHDRS
{
    text    PT_LOAD    FLAGS((1 << 0) | (1 << 2)) ; /* Execute + Read */
    rodata  PT_LOAD    FLAGS((1 << 2)) ;            /* Read only */
    data    PT_LOAD    FLAGS((1 << 1) | (1 << 2)) ; /* Write + Read */
    dynamic PT_DYNAMIC FLAGS((1 << 1) | (1 << 2)) ; /* Dynamic PHDR for relocations */
}

/* Define the sections we want */
SECTIONS
{
    /* We wanna be placed in the topmost 2GiB of the address space, for optimisations */
    /* and because that is what the Limine spec mandates. */
    /* Any address in this region will do, but often 0xFFFFFFFF80000000 is chosen as */
    /* that is the beginning of the region. */
    . = 0xFFFFFFFF80000000;

    .text : {
        *(.text.boot)
        *(.__init)
        . += CONSTANT(MAXPAGESIZE);
        *(.text .text.*)
    } :text

    . += CONSTANT(MAXPAGESIZE);

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    . += CONSTANT(MAXPAGESIZE);

    .data : ALIGN(4096) {
        *(.data .data.*)
        *(.sdata .sdata.*)

        KEEP(*(.requests_start))
        KEEP(*(.requests))
        KEEP(*(.requests_end))

        KEEP(*(SORT_BY_NAME(.__init_data.*)))

        PROVIDE_HIDDEN(__global_pointer$ = . + 0x800);
    } :data

    .dynamic : ALIGN(4096) {
        *(.dynamic)
    } :data :dynamic

    /*
     * Early-boot scratch (CPU map snapshot, …). Collected before catch-all
     * `.bss.*` so it is not swallowed into permanent BSS. Page-aligned for
     * PMM free of [__boot_reclaim_start, __boot_reclaim_end) after BootInfo
     * is consumed (do not keep &'static into this range past that point).
     */
    .boot_reclaim (NOLOAD) : ALIGN(4096) {
        __boot_reclaim_start = .;
        KEEP(*(.boot_reclaim .boot_reclaim.*))
        KEEP(*(.bss.reclaim .bss.reclaim.*))
        KEEP(*(.data.reclaim .data.reclaim.*))
        . = ALIGN(4096);
        __boot_reclaim_end = .;
    } :data

    .bss : ALIGN(4096) {
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        *(COMMON)
    } :data

    . = ALIGN(4096);

    __end = .;

    /* Discard .note.* and .eh_frame since they may cause issues on some hosts. */
    /DISCARD/ : {
        *(.comment*)
        *(.eh_frame*)
        *(.gcc_except_table*)
        *(.note*)
        *(.rel.eh_frame*)
    }
}