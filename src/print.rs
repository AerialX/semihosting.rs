use core::fmt;
use core::num::NonZeroUsize;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::io::{Handle, OwnedHandle, write_char, open};
use crate::Mode;
use cstrptr::cstr;

/// A slow but simple debugging interface
pub struct CharPrinter;

impl fmt::Write for CharPrinter {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        fmt::Write::write_str(&mut &*self, s)
    }
}

impl<'a> fmt::Write for &'a CharPrinter {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print_str(s);
        Ok(())
    }
}

#[cfg(feature = "ufmt-write")]
impl ufmt_write::uWrite for CharPrinter {
    type Error = core::convert::Infallible;

    #[inline]
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        ufmt_write::uWrite::write_str(&mut &*self, s)
    }
}

#[cfg(feature = "ufmt-write")]
impl<'a> ufmt_write::uWrite for &'a CharPrinter {
    type Error = <CharPrinter as ufmt_write::uWrite>::Error;

    #[inline]
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
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

#[cfg(feature = "enable-logger")]
pub static LOGGER: GlobalLogger = GlobalLogger::new();
#[cfg(not(feature = "enable-logger"))]
pub static LOGGER: CharPrinter = CharPrinter;

pub struct GlobalLogger {
    handle: AtomicUsize,
}

#[cfg(feature = "const-default")]
impl const_default::ConstDefault for GlobalLogger {
    const DEFAULT: Self = GlobalLogger::new();
}

impl GlobalLogger {
    #[inline]
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

    #[inline]
    pub fn log(&self, str: &str) -> Option<()> {
        self.handle()?.write_all::<()>(str.as_bytes()).ok()
    }

    pub fn into_handle(self) -> Option<OwnedHandle> {
        let handle = self.handle();
        handle.map(OwnedHandle::from_handle)
    }
}

impl<'a> fmt::Write for &'a GlobalLogger {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.log(s).ok_or(fmt::Error)
    }
}

#[cfg(feature = "ufmt-write")]
impl<'a> ufmt_write::uWrite for &'a GlobalLogger {
    type Error = (); // TODO: return syscall error code here instead?

    #[inline]
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.log(s).ok_or(())
    }
}
