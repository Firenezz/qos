pub trait Processor {
    fn id(&self) -> u64;
    fn acpi_uid(&self) -> u32;
    fn is_bsp(&self) -> bool;
}
