use alloc::vec::Vec;
use smallvec::SmallVec;


#[derive(Clone, Copy, Debug)]
enum BootloaderMemoryKind {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNVS,
    BadMemory,
}

pub trait ExternalMemoryEntry {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn kind(&self) -> BootloaderMemoryKind;
}

#[derive(Clone, Copy, Debug)]
struct MemoryEntry {
    pub start: usize,
    pub end: usize,
    pub kind: BootloaderMemoryKind,
}

struct MemoryMap {
    boot_map: [MemoryEntry; 128],
    runtime_map: SmallVec<[MemoryEntry; 32]>,
    size: usize,
}