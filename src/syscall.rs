#[macro_export]
macro_rules! syscall {
    ($syscall:expr) => {
        $crate::syscall0($syscall)
    };
    ($syscall:expr $(,$arg:expr)+) => {
        // TODO this won't work, use a tuple struct instead?
        $crate::syscall($syscall, [$($arg as usize,)*].as_ptr() as usize)
    };
}

pub unsafe fn syscall0<S: Into<usize>>(syscall: S) -> usize {
    self::syscall::<_, usize>(syscall, 0)
}

#[cfg(any(thumb, arm))]
pub unsafe fn syscall<S: Into<usize>, T: Into<usize>>(syscall: S, message: T) -> usize {
    // note on clobbers:
    // - memory is complicated depending on the operation? though indirect pointers mean this still may not be enough hence "volatile"
    // - lr clobbered if in supervisor mode? newlib says so... see "page 13-77 of ARM DUI 0040D"
    // maybe plan on using a macro or per-syscall functions for this..? though I guess we do want this to be shared...
    #[cfg(not(feature = "v2"))]
    unsafe fn syscall_impl(syscall: usize, message: usize) -> usize {
        #[cfg(not(thumb))]
        const SVC: usize = 0x123456;
        #[cfg(thumb)]
        const SVC: usize = 0xab;

        #[cfg(any(not(thumb), not(any(target = "thumbv6m-none-eabi", target = "thumbv7m-none-eabi"))))]
        macro_rules! syscall_asm { () => { "svc $1" } };
        #[cfg(all(thumb, any(target = "thumbv6m-none-eabi", target = "thumbv7m-none-eabi")))]
        macro_rules! syscall_asm { () => { "bkpt $1" } };

        let out: usize;
        #[cfg(feature = "unstable")]
        asm!(syscall_asm!()
            : "={r0}"(out)
            : "i"(SVC), "0"(syscall), "{r1}"(message)
            : "memory" , "lr"
            : "volatile"
        );
        #[cfg(not(feature = "unstable"))]
        compile_error!("external asm unimplemented");
        out
    }


    #[cfg(feature = "v2")]
    unsafe fn syscall_impl(syscall: usize, message: usize) -> usize {
        let out: usize;
        // TODO is this necessary over using hlt? this may only be a problem with old assemblers, and if we're using llvm...
        #[cfg(not(thumb))]
        macro_rules! syscall_asm { () => { ".inst 0xE10F0070" } }; // HLT #0xF000
        #[cfg(thumb)]
        macro_rules! syscall_asm { () => { ".inst 0xBABC" } }; // HLT #0x3c

        #[cfg(feature = "unstable")]
        asm!(syscall_asm!()
            : "={r0}"(out)
            : "0"(syscall), "{r1}"(message)
            : "memory" , "lr"
            : "volatile"
        );
        #[cfg(not(feature = "unstable"))]
        compile_error!("external asm unimplemented");

        out
    }

    syscall_impl(syscall.into(), message.into())
}

#[cfg(not(any(thumb, arm)))]
pub unsafe fn syscall<S: Into<usize>, T: Into<usize>>(_syscall: S, _message: T) -> usize {
    unimplemented!("stub")
}
