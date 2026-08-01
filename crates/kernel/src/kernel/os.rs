use crate::kernel::cpu::ProcessorMap;

/// Contains Kernel State and manages the kernel's resources
pub struct Os {
    /// Map of processors in the system
    processor_map: ProcessorMap,

}

impl Os {
    pub fn new() -> Self {
        Self { processor_map: ProcessorMap::new() }
    }

    pub fn register_processor(&mut self, processor: crate::boot::CpuInfo) {
        self.processor_map.add_into(processor);
    }
}