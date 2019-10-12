#![cfg_attr(feature = "unstable", feature(asm, core_intrinsics))]
#![no_std]

use core::num::NonZeroU32;
use cstrptr::CStr;

mod macros;

mod export;
mod syscall;
pub mod io;
pub mod print;

#[doc(hidden)]
pub mod _export {
    pub use super::export::*;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(usize)]
pub enum Syscall {
    Open = 1,
    Close = 2,
    WriteC = 3,
    Write0 = 4,
    Write = 5,
    Read = 6,
    ReadC = 7,
    IsError = 8,
    IsTTY = 9,
    Seek = 10,

    FLen = 12,
    TmpNam = 13,
    Remove = 14,
    Rename = 15,
    Clock = 16,
    Time = 17,
    System = 18,
    Errno = 19,

    GetCmdline = 21,
    HeapInfo = 22,

    EnterSVC = 23,
    ReportException = 24,
    #[cfg(feature = "v2")]
    ReportExceptionExtended = 32,

    Elapsed = 48,
    TickFreq = 49,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(usize)]
pub enum Exception {
    // Hardware vector reason codes
    BranchThroughZero = 0x20000,
    UndefinedInstr = 0x20001,
    SoftwareInterrupt = 0x20002,
    PrefetchAbort = 0x20003,
    DataAbort = 0x20004,
    AddressException = 0x20005,
    IRQ = 0x20006,
    FIQ = 0x20007,

    // Software reason codes
    BreakPoint = 0x20020,
    WatchPoint = 0x20021,
    StepComplete = 0x20022,
    RunTimeErrorUnknown = 0x20023,
    InternalError = 0x20024,
    UserInterruption = 0x20025,
    ApplicationExit = 0x20026,
    StackOverflow = 0x20027,
    DivisionByZero = 0x20028,
    OSSpecific = 0x20029,
}

impl From<Syscall> for usize {
    fn from(s: Syscall) -> Self {
        s as _
    }
}

impl From<Exception> for usize {
    fn from(s: Exception) -> Self {
        s as _
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct HeapInfo {
    heap_base: Option<NonZeroU32>,
    heap_limit: Option<NonZeroU32>,
    stack_base: Option<NonZeroU32>,
    stack_limit: Option<NonZeroU32>,
}

bitflags::bitflags! {
    pub struct Mode: u32 {
        const BINARY = 1;
        const MODE_READ_ONLY = 0;
        const MODE_READ_WRITE = 4;
        const MODE_APPEND = 8;
    }
}

bitflags::bitflags! {
    pub struct Extensions: u32 {
        const EXIT_EXTENDED = 1;
        const STDOUT_STDERR = 2;
    }
}

const MAGIC: &'static [u8] = b"SHFB"; // TODO check endianness

pub use syscall::{syscall, syscall0};
pub use print::{CharPrinter, print_str, print_cstr, print_char};

/// Normal application exit
///
/// Typically results in a successful 0 exit code.
pub fn exit() -> ! {
    unsafe { exit_with(Exception::ApplicationExit) }
}

/// Abnormal application exit
///
/// Typically results in an exit code of 1.
pub fn abort() -> ! {
    unsafe { exit_with(Exception::InternalError) } // or RunTimeErrorUnknown? OSSpecific?
}

/// Abort with the given exception reason
#[inline]
pub unsafe fn exit_with(exception: Exception) -> ! {
    loop {
        io::report_exception(exception);
        #[cfg(not(debug_assertions))]
        core::hint::unreachable_unchecked();
    }
}

pub fn parse_cmdline<T, F: for<'a> FnOnce(&'a CStr) -> T>(f: F) -> Result<T, ()> {
    use core::mem::{MaybeUninit, transmute};
    unsafe {
        unsafe fn transmute_slice<'a, A, B>(s: &'a [A]) -> &'a [B] {
            // ensure that we don't mess up the lifetime
            transmute(s)
        }

        let mut buffer: [MaybeUninit<u8>; 80] = MaybeUninit::uninit().assume_init();
        let len = io::get_cmdline_unchecked(buffer[..].as_mut_ptr() as *mut _, buffer.len())?;

        // impossible since usize::MAX is already interpreted as an Err()
        #[cfg(feature = "unstable")]
        let buf_len = core::intrinsics::unchecked_add(len, 1);
        #[cfg(not(feature = "unstable"))]
        let buf_len = len + 1;

        let slice = match buffer.get(..buf_len) {
            Some(slice) => slice,
            #[cfg(debug_assertions)]
            None => unreachable!("cmdline buffer length invalid"),
            #[cfg(not(debug_assertions))]
            None => core::hint::unreachable_unchecked(),
        };
        let slice = transmute_slice::<_, u8>(slice);
        Ok(f(CStr::from_bytes_with_nul_unchecked(slice)))
    }
}
