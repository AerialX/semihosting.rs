use core::fmt;
use core::num::NonZeroUsize;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::io::{Handle, OwnedHandle, write_char, open};
use crate::Mode;
use cstrptr::cstr;

/// A slow but simple debugging interface
pub struct CharPrinter;

impl fmt::Write for CharPrinter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print_str(s);
        Ok(())
    }
}

pub fn print_str(str: &str) {
    for c in str.as_bytes() {
        write_char(*c)
    }
}

pub use crate::io::{write_cstr as print_cstr, write_char as print_char};

#[test]
fn print() {
    use crate::print;
    print!("hi");
    print!("{}", 5);
}

pub struct GlobalLogger {
    handle: AtomicUsize,
}

impl GlobalLogger {
    pub const fn new() -> Self {
        Self {
            handle: AtomicUsize::new(0),
        }
    }

    pub fn handle(&self) -> Option<Handle> {
        Some(match NonZeroUsize::new(self.handle.load(Ordering::Relaxed)) {
            None => {
                let fd = open::<()>(cstr!(":tt"), Mode::MODE_APPEND).ok()?;
                self.handle.store(fd.get(), Ordering::Relaxed);
                Handle::from_fd(fd)
            },
            Some(fd) => Handle::from_fd(fd),
        })
    }

    pub fn log(&self, str: &str) -> Option<()> {
        self.handle()?.write_all::<()>(str.as_bytes()).ok()
    }

    pub fn into_handle(self) -> Option<OwnedHandle> {
        let handle = self.handle();
        handle.map(OwnedHandle::from_handle)
    }
}
