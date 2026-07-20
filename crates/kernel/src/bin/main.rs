//! qos-kernel image (Limine by default).
//!
//! Boot path: `boot_main` → `early_start` → `limine::handoff` → `kernel_main`

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(::qos_kernel::test_runner)]


#[cfg(feature = "boot-limine")]
mod limine_image {
    use limine::{RequestsEndMarker, RequestsStartMarker};

    /// Pull Limine request objects (including `ENTRY_POINT_REQUEST` → `boot_main`).
    #[used]
    static _KEEP_ENTRY_POINT_REQUEST: &limine::request::EntryPointRequest =
        &qos_kernel::boot::limine::ENTRY_POINT_REQUEST;

    #[used]
    #[unsafe(link_section = ".requests_start")]
    static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

    #[used]
    #[unsafe(link_section = ".requests_end")]
    static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();
}
