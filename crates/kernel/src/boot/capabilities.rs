


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityType {
    Cpu,
    Memory,
    Io,
    Pci,
    Uart,
    Gpio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability<T> {
    Present(T),
    Absent,
}

impl<T> Capability<T> {
    pub fn is_present(&self) -> bool {
        matches!(self, Capability::Present(_))
    }

    pub fn is_absent(&self) -> bool {
        matches!(self, Capability::Absent)
    }
}

impl<T> From<Option<T>> for Capability<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Capability::Present(value),
            None => Capability::Absent,
        }
    }
}

impl<T> From<Capability<T>> for Option<T> {
    fn from(value: Capability<T>) -> Self {
        match value {
            Capability::Present(value) => Some(value),
            Capability::Absent => None,
        }
    }
}