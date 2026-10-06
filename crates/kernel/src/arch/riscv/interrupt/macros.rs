/// Save GPRs + `sepc`. Skips `sp` (entry saves it) and `tp` (TLS lives in
/// `sscratch` for the duration of the handler; entry swaps it back).
#[macro_export]
macro_rules! push_registers {
    () => {
        "
    addi    sp, sp, -32 * 8
    sd      x1, (0 * 8)(sp)
    // skip sp
    sd      x3, (2 * 8)(sp)
    // skip tp
    sd      x5, (4 * 8)(sp)
    sd      x6, (5 * 8)(sp)
    sd      x7, (6 * 8)(sp)
    sd      x8, (7 * 8)(sp)
    sd      x9, (8 * 8)(sp)
    sd      x10, (9 * 8)(sp)
    sd      x11, (10 * 8)(sp)
    sd      x12, (11 * 8)(sp)
    sd      x13, (12 * 8)(sp)
    sd      x14, (13 * 8)(sp)
    sd      x15, (14 * 8)(sp)
    sd      x16, (15 * 8)(sp)
    sd      x17, (16 * 8)(sp)
    sd      x18, (17 * 8)(sp)
    sd      x19, (18 * 8)(sp)
    sd      x20, (19 * 8)(sp)
    sd      x21, (20 * 8)(sp)
    sd      x22, (21 * 8)(sp)
    sd      x23, (22 * 8)(sp)
    sd      x24, (23 * 8)(sp)
    sd      x25, (24 * 8)(sp)
    sd      x26, (25 * 8)(sp)
    sd      x27, (26 * 8)(sp)
    sd      x28, (27 * 8)(sp)
    sd      x29, (28 * 8)(sp)
    sd      x30, (29 * 8)(sp)
    sd      x31, (30 * 8)(sp)

    csrr    t0, sepc
    sd      t0, (31 * 8)(sp)
    "
    }; // keep sepc value in t0 on exit
}

#[macro_export]
macro_rules! pop_registers {
    () => {
        "
    ld      t0, (31 * 8)(sp)
    csrw    sepc, t0

    ld      x1, (0 * 8)(sp)
    // skip sp, it'll be restored later
    ld      x3, (2 * 8)(sp)
    // skip tp (restored via sscratch swap in entry)
    ld      x5, (4 * 8)(sp)
    ld      x6, (5 * 8)(sp)
    ld      x7, (6 * 8)(sp)
    ld      x8, (7 * 8)(sp)
    ld      x9, (8 * 8)(sp)
    ld      x10, (9 * 8)(sp)
    ld      x11, (10 * 8)(sp)
    ld      x12, (11 * 8)(sp)
    ld      x13, (12 * 8)(sp)
    ld      x14, (13 * 8)(sp)
    ld      x15, (14 * 8)(sp)
    ld      x16, (15 * 8)(sp)
    ld      x17, (16 * 8)(sp)
    ld      x18, (17 * 8)(sp)
    ld      x19, (18 * 8)(sp)
    ld      x20, (19 * 8)(sp)
    ld      x21, (20 * 8)(sp)
    ld      x22, (21 * 8)(sp)
    ld      x23, (22 * 8)(sp)
    ld      x24, (23 * 8)(sp)
    ld      x25, (24 * 8)(sp)
    ld      x26, (25 * 8)(sp)
    ld      x27, (26 * 8)(sp)
    ld      x28, (27 * 8)(sp)
    ld      x29, (28 * 8)(sp)
    ld      x30, (29 * 8)(sp)
    ld      x31, (30 * 8)(sp)
    ld      sp, (1 * 8)(sp)
    "
    };
}
