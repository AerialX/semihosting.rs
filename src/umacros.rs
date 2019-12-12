#[doc(hidden)]
#[macro_export]
macro_rules! uprint_fmt {
    ($($tt:tt)*) => {
        {
            use $crate::_export::{
                ufmt_write::uWrite,
                ufmt,
            };
            // failures aren't interesting to us
            let _ = $crate::_export::ufmt::uwrite!(&$crate::LOGGER, $($tt)*);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! uprintln_fmt {
    ($($tt:tt)*) => {
        {
            use $crate::_export::{
                ufmt_write::uWrite,
                ufmt,
            };
            // failures aren't interesting to us
            let _ = $crate::_export::ufmt::uwriteln!(&$crate::LOGGER, $($tt)*);
        }
    };
}

#[doc(hidden)]
#[cfg(feature = "enable-logger")]
#[macro_export]
macro_rules! uprint_str {
    ($str:expr) => {
        {
            let _ = $crate::_export::ufmt_write::uWrite::write_str(&mut &$crate::LOGGER, $str);
        }
    };
}

#[doc(hidden)]
#[cfg(feature = "enable-logger")]
#[macro_export]
macro_rules! uprintln_str {
    ($str:expr) => {
        {
            let _ = $crate::_export::ufmt_write::uWrite::write_str(&mut &$crate::LOGGER, $str);
            let _ = $crate::_export::ufmt_write::uWrite::write_str(&mut &$crate::LOGGER, "\n");
        }
    };
}

#[doc(hidden)]
#[cfg(not(feature = "enable-logger"))]
#[macro_export]
macro_rules! uprint_str {
    ($str:literal) => {
        $crate::print_cstr($crate::_export::cstr!($str).into())
    };
    ($str:expr) => {
        $crate::uprint_fmt!("{}", $str)
    };
}

#[doc(hidden)]
#[cfg(not(feature = "enable-logger"))]
#[macro_export]
macro_rules! uprintln_str {
    ($str:literal) => {
        $crate::print_cstr($crate::_export::cstr!(concat!($str, "\n")).into())
    };
    ($str:expr) => {
        $crate::uprintln_fmt!("{}", $str)
    };
}

#[macro_export]
macro_rules! uprint {
    ($str:literal) => {
        $crate::uprint_str!($str)
    };
    ($str:expr) => {
        $crate::uprint_str!($str)
    };
    ($($tt:tt)*) => {
        $crate::uprint_fmt!($($tt)*)
    };
}

#[macro_export]
macro_rules! uprintln {
    ($str:literal) => {
        $crate::uprintln_str!($str)
    };
    ($str:expr) => {
        $crate::uprintln_str!($str)
    };
    ($($tt:tt)*) => {
        $crate::uprintln_fmt!($($tt)*)
    };
}
