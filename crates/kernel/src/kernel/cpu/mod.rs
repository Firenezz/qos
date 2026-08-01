use hashbrown::HashMap;
use thin_vec::ThinVec;

/// This is the information to identify a processor.
/// 
#[derive(Debug, Clone, Copy)]
pub struct ProcessorInfo {
    /// Dense kernel index [0..n], assigned by the kernel.
    pub id: usize,
    /// Hardware ID, assigned by the hardware. (hartid/LAPIC ID/etc)
    pub hw_id: usize,
    /// Firmware ID, assigned by the firmware. (ACPI Processor UID/etc)
    pub firmware_id: Option<u64>,
    /// Whether the processor is online.
    pub is_online: bool,
    /// Whether the processor is enabled. (APs are disabled by default)
    pub enabled: bool,
}


impl ProcessorInfo {
    pub fn new(id: usize, hw_id: usize, firmware_id: Option<u64>) -> Self {
        Self { id, hw_id, firmware_id, is_online: false, enabled: false }
    }
}

impl Default for ProcessorInfo {
    fn default() -> Self {
        Self::new(0, 0, None)
    }
}

/// This is the map of processors in the system.
/// 
/// It is used by the kernel to identify and manage the processors in the system.
#[derive(Debug, Clone)]
pub struct ProcessorMap {
    pub processors: ThinVec<Option<ProcessorInfo>>,
    hw_id_to_index: HashMap<u64, usize>,
    firmware_id_to_index: HashMap<u64, usize>,
}

impl ProcessorMap {
    pub fn new() -> Self {
        Self { processors: ThinVec::new(), hw_id_to_index: HashMap::new(), firmware_id_to_index: HashMap::new() }
    }

    pub fn add_into(&mut self, boot_info: impl Into<ProcessorInfo>) -> ProcessorInfo {
        let processor = boot_info.into();
        self.add_processor(processor);
        processor
    }

    pub fn add_processor(&mut self, processor: ProcessorInfo) {
        self.processors.push(Some(processor));
        self.hw_id_to_index.insert(processor.hw_id as u64, self.processors.len() - 1);
        if let Some(firmware_id) = processor.firmware_id {
            self.firmware_id_to_index.insert(firmware_id, self.processors.len() - 1);
        }
    }

    pub fn get_processor_by_id(&self, id: usize) -> Option<&ProcessorInfo> {
        self.processors.get(id).and_then(|processor| processor.as_ref())
    }

    pub fn get_processor_by_hw_id(&self, hw_id: u64) -> Option<&ProcessorInfo> {
        self.hw_id_to_index.get(&hw_id).and_then(|index| self.processors.get(*index).and_then(|processor| processor.as_ref()))
    }

    pub fn get_processor_by_firmware_id(&self, firmware_id: u64) -> Option<&ProcessorInfo> {
        self.firmware_id_to_index.get(&firmware_id).and_then(|index| self.processors.get(*index).and_then(|processor| processor.as_ref()))
    }

    /// Find the next free ID in the map.
    /// 
    /// This loops through all processors but this will not be used very often (i.e. on boot and hotplug)
    pub fn find_next_free_id(&self) -> usize {
        self.processors.iter().enumerate().find(|(_, processor)| processor.is_none()).map(|(id, _)| id).unwrap_or(self.processors.len())
    }
}
