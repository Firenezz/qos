//! Global early console and `log` backend.
//!
//! Arch code constructs a platform [`Uart`] and passes it to [`init`]; this
//! module owns locking and the logger.

use core::fmt::Write;

use spin::Mutex;

use crate::arch::device::uart::Uart;
use crate::arch::{Arch, Current};

static CONSOLE: Mutex<Option<Uart>> = Mutex::new(None);

/// Run `f` with exclusive access to the console UART.
///
/// Takes the spin lock for cross-hart exclusion, and masks IRQs on this hart so a
/// nested log from trap context cannot deadlock on the same lock.
fn with_console<R>(f: impl FnOnce(&mut Option<Uart>) -> R) -> R {
    // Mask IRQs before taking the lock so a trap cannot re-enter and deadlock.
    Current::with_disabled_irqs(|| {
        let mut guard = CONSOLE.lock();
        f(&mut *guard)
    })
}

#[allow(unused)]
struct UartLogger;

impl log::Log for UartLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        with_console(|console| {
            if let Some(uart) = console.as_mut() {
                match record.level() {
                    log::Level::Debug | log::Level::Trace | log::Level::Error => {
                        let _ = write!(uart, "line {}, module {}: [{}] {}", record.line().unwrap(), record.module_path().unwrap(), record.level(), record.args());
                    }
                    level => {
                        let _ = write!(uart, "[{}] {}", level, record.args());
                    }
                }
            }
        });
    }

    fn flush(&self) {}
}

#[allow(unused)]
struct ColoredUartLogger;

impl log::Log for ColoredUartLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }
    
    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        with_console(|console| {
            let color = match record.level() {
                log::Level::Error => "\x1b[31m",
                log::Level::Warn => "\x1b[33m",
                log::Level::Info => "\x1b[32m",
                log::Level::Debug => "\x1b[34m",
                log::Level::Trace => "\x1b[35m",
            };
            let reset = "\x1b[0m";
            if let Some(uart) = console.as_mut() {
                match record.level() {
                    log::Level::Debug | log::Level::Trace | log::Level::Error => {
                        let _ = writeln!(uart, "line {}, module {}: {}[{}] {}{}", record.line().unwrap(), record.module_path().unwrap(), color, record.level(), record.args(), reset);
                    }
                    level => {
                        let _ = writeln!(uart, "{}[{}] {}{}", color, level, record.args(), reset);
                    }
                }
            }
        });
    }

    fn flush(&self) {} // no need to flush
}

static LOGGER: ColoredUartLogger = ColoredUartLogger;

/// Install `uart` as the global console and `log` backend.
pub fn init(uart: Uart) {
    uart.init();

    with_console(|console| {
        *console = Some(uart);
    });

    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Trace);
}
