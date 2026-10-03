pub mod memory;

pub use memory::{
    init_from_boot, promote_through, register_device_memory, with_memory, with_memory_mut,
};
