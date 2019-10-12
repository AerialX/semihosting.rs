use core::num::NonZeroUsize;
use core::mem::{MaybeUninit, forget};
use core::ops::{Deref, DerefMut};
use cstrptr::{CStr, CStrPtr};
use crate::{ syscall, Syscall, Exception, HeapInfo, Mode };

#[cfg(not(feature = "v2"))]
pub fn features() -> ! {
    unimplemented!("stdout/stderr and no exit extended");
}

#[cfg(feature = "v2")]
pub fn features() -> Handle {
    let fd = Handle::open(":semihosting-features", O_RDONLY)?;
    let len = fd.flen()?;
    if len < MAGIC.len() {
        panic!("file too small")
    }
    // assert file format is:
    // 1. MAGIC (4 ascii bytes)
    // 2. rest of file is a bitfield? flen tells you how long I guess, but another function can read into that?
    unimplemented!()
}

#[derive(Copy, Clone, Debug)]
pub struct Handle {
    fd: NonZeroUsize,
}

#[derive(Copy, Clone, Debug)]
pub enum WriteAllError<E> {
    Io(E),
    Incomplete(usize),
    Invalid, // syscall returned nonsense? this would be an assertion but... don't want to panic!
}

impl Handle {
    #[inline]
    pub fn open<E: Errno>(path: &CStr, mode: Mode) -> Result<Self, E> {
        open(path, mode).map(Handle::from_fd)
    }

    #[inline]
    pub const fn from_fd(fd: NonZeroUsize) -> Self {
        // TODO should this be unsafe?
        Handle {
            fd
        }
    }

    #[inline]
    pub const fn fd(&self) -> NonZeroUsize {
        self.fd
    }

    #[inline]
    pub fn seek_set<E: Errno>(&self, offset: usize) -> Result<(), E> {
        seek(self.fd.get(), offset).map(drop)
    }

    #[inline]
    pub fn close<E: Errno>(self) -> Result<(), E> {
        close(self.fd.get())
    }

    #[inline]
    pub fn write<E: Errno>(&self, data: &[u8]) -> Result<usize, E> {
        write(self.fd.get(), data)
    }

    #[inline]
    pub fn read<E: Errno>(&self, data: &mut [u8]) -> Result<usize, E> {
        read(self.fd.get(), data)
    }

    pub fn write_all<E: Errno>(&self, mut buffer: &[u8]) -> Result<(), WriteAllError<E>> {
        while !buffer.is_empty() {
            match self.write(buffer).map_err(WriteAllError::Io)? {
                0 => return Ok(()),
                left if left == buffer.len() => return Err(WriteAllError::Incomplete(left)),
                left => {
                    buffer = match buffer.len().checked_sub(left).and_then(|off| buffer.get(off..)) {
                        Some(buffer) => buffer,
                        None => return Err(WriteAllError::Invalid),
                    }
                },
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct OwnedHandle {
    handle: Handle,
}

impl OwnedHandle {
    #[inline]
    pub fn open<E: Errno>(path: &CStr, mode: Mode) -> Result<Self, E> {
        Ok(Self {
            handle: Handle::open(path, mode)?,
        })
    }

    #[inline]
    pub const fn from_handle(handle: Handle) -> Self {
        // TODO should this be unsafe?
        Self {
            handle,
        }
    }

    #[inline]
    pub fn into_handle(self) -> Handle {
        let handle = self.handle.clone();
        forget(self);
        handle
    }
}

impl Deref for OwnedHandle {
    type Target = Handle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl DerefMut for OwnedHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            if let Err(e) = self.close::<isize>() {
                // TODO log error
            }
        }

        #[cfg(not(debug_assertions))]
        let _ = self.close::<()>();
    }
}

pub trait Errno {
    fn last_error() -> Self;
}

impl Errno for () {
    #[inline]
    fn last_error() -> Self { }
}

impl Errno for isize {
    #[inline]
    fn last_error() -> Self {
        errno()
    }
}

#[inline]
pub fn errno() -> isize {
    unsafe { syscall!(Syscall::Errno) as isize }
}

#[inline]
fn map_res<E: Errno>(res: usize) -> Result<usize, E> {
    match res {
        core::usize::MAX => Err(E::last_error()),
        res => Ok(res)
    }
}

#[inline]
pub fn system<E: Errno>(cmd: &CStr) -> Result<usize, E> {
    let cmd = cmd.to_bytes();
    map_res(unsafe { syscall!(Syscall::System, cmd.as_ptr(), cmd.len()) })
}

/// Buffer size must be at least L_tmpnam on the host system (20 for glibc?)
#[inline]
pub fn tmpnam(id: u8, buffer: &mut [u8]) -> Result<(), ()> {
    map_res(unsafe { syscall!(Syscall::TmpNam, buffer.as_mut_ptr(), id, buffer.len()) })
        .map(drop)
}

#[inline]
pub fn remove<E: Errno>(path: &CStr) -> Result<usize, E> {
    let len = path.to_bytes().len(); // NOTE: not guaranteed to be zero-cost
    map_res(unsafe { syscall!(Syscall::Remove, path.as_ptr(), len) })
}

#[inline]
pub fn rename<E: Errno>(src: &CStr, dest: &CStr) -> Result<usize, E> {
    map_res(unsafe { syscall!(Syscall::Rename,
        src.as_ptr(), src.to_bytes().len(),
        dest.as_ptr(), dest.to_bytes().len()
    ) })
}

#[inline]
pub fn open<E: Errno>(path: &CStr, mode: Mode) -> Result<NonZeroUsize, E> {
    let len = path.to_bytes().len();
    unsafe {
        map_res(syscall!(Syscall::Open, path.as_ptr(), mode.bits(), len)).map(|fd| match NonZeroUsize::new(fd) {
            // not allowed by the semihosting spec, but should we guard against noncompliant implementations?
            #[cfg(debug_assertions)]
            None => unreachable!("invalid open result"),
            #[cfg(not(debug_assertions))]
            None => core::hint::unreachable_unchecked(),
            Some(fd) => fd,
        })
    }
}

/// Returns number of bytes that were *not* read
#[inline]
pub fn read<E: Errno>(fd: usize, data: &mut [u8]) -> Result<usize, E> {
    map_res(unsafe { syscall!(Syscall::Read, fd, data.as_mut_ptr(), data.len()) })
}

/// Returns number of bytes that were *not* written
#[inline]
pub fn write<E: Errno>(fd: usize, data: &[u8]) -> Result<usize, E> {
    map_res(unsafe { syscall!(Syscall::Write, fd, data.as_ptr(), data.len()) })
}

/// Seek to the specified absolute position.
///
/// Seeking out of bounds is undefined, use [`flen`] to avoid this.
#[inline]
pub fn seek<E: Errno>(fd: usize, offset: usize) -> Result<(), E> {
    // TODO docs say negative value on failure, not necessarily -1?
    map_res(unsafe { syscall!(Syscall::Seek, fd, offset) })
        .map(drop)
}

#[inline]
pub fn f_len<E: Errno>(fd: usize) -> Result<usize, E> {
    map_res(unsafe { syscall!(Syscall::FLen, fd) })
}

#[inline]
pub fn close<E: Errno>(fd: usize) -> Result<(), E> {
    map_res(unsafe { syscall!(Syscall::Close, fd) }).map(drop)
}

#[inline]
pub fn is_tty<E: Errno>(fd: usize) -> Result<(), E> {
    match unsafe { syscall!(Syscall::IsTTY, fd) } {
        1 => Ok(()),
        _ => Err(E::last_error()),
    }
}

/// Seconds since Unix epoch
#[inline]
pub fn time() -> usize {
    unsafe { syscall!(Syscall::Time) }
}

/// 100Hz clock ticks
#[inline]
pub fn clock() -> Result<usize, ()> {
    map_res(unsafe { syscall!(Syscall::Clock) })
}

#[inline]
pub fn tick_freq() -> Result<usize, ()> {
    map_res(unsafe { syscall!(Syscall::TickFreq) })
}

#[inline]
/// Get Commandline arguments.
///
/// Returns the length of the retrieved string. Agents should be able to
/// transfer at least 80 bytes.
pub fn get_cmdline(buffer: &mut [u8]) -> Result<usize, ()> {
    unsafe { get_cmdline_unchecked(buffer.as_mut_ptr(), buffer.len()) }
}

#[inline]
pub unsafe fn get_cmdline_unchecked(buffer: *mut u8, len: usize) -> Result<usize, ()> {
    let mut block = [ buffer as usize, len ];
    map_res(syscall(Syscall::GetCmdline, block.as_mut_ptr() as usize))
        .map(|_| *block.get_unchecked(1))
}

#[inline]
pub fn is_error(res: usize) -> bool {
    unsafe { syscall!(Syscall::IsError, res) != 0 }
}

#[inline]
pub fn read_char() -> u8 {
    unsafe { syscall!(Syscall::ReadC) as u8 }
}

#[inline]
pub fn write_char(char: u8) {
    unsafe { syscall(Syscall::WriteC, &char as *const _ as usize) };
}

#[inline]
pub fn write_cstr(str: CStrPtr) {
    unsafe { syscall(Syscall::Write0, str.as_ptr() as usize) };
}

/// Report an exception or exit condition to the debugger
///
/// Although typically not resumable, the debugger can choose to continue.
#[inline]
pub fn report_exception(reason: Exception) -> usize {
    unsafe { syscall(Syscall::ReportException, reason) }
}

#[inline]
pub fn heapinfo() -> HeapInfo {
    let mut info = MaybeUninit::uninit();
    unsafe {
        syscall(Syscall::HeapInfo, info.as_mut_ptr() as usize);
        info.assume_init()
    }
}
