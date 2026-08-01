

pub struct Paging {
    /// The kernel's page table
    page_table: PageTable,
}

impl Paging {
    pub fn new() -> Self {
        Self { page_table: PageTable::new() }
    }
}