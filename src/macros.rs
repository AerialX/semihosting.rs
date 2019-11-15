#[doc(hidden)]
#[macro_export]
macro_rules! print_fmt {
    ($($tt:tt)*) => {
        {
            use $crate::_export::core::fmt::Write;
            // failures aren't interesting to us
            let _ = $crate::_export::core::write!(&$crate::LOGGER, $($tt)*);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! println_fmt {
    ($($tt:tt)*) => {
        {
            use $crate::_export::core::fmt::Write;
            // failures aren't interesting to us
            let _ = $crate::_export::core::writeln!(&$crate::LOGGER, $($tt)*);
        }
    };
}

#[doc(hidden)]
#[cfg(feature = "enable-logger")]
#[macro_export]
macro_rules! print_str {
    ($str:expr) => {
        {
            let _ = $crate::_export::core::fmt::Write::write_str(&mut &$crate::LOGGER, $str);
        }
    };
}

#[doc(hidden)]
#[cfg(feature = "enable-logger")]
#[macro_export]
macro_rules! println_str {
    ($str:expr) => {
        {
            let _ = $crate::_export::core::fmt::Write::write_str(&mut &$crate::LOGGER, $str);
            let _ = $crate::_export::core::fmt::Write::write_str(&mut &$crate::LOGGER, "\n");
        }
    };
}

#[doc(hidden)]
#[cfg(not(feature = "enable-logger"))]
#[macro_export]
macro_rules! print_str {
    ($str:literal) => {
        $crate::print_cstr($crate::_export::cstr!($str).into())
    };
    ($str:expr) => {
        $crate::print_fmt!("{}", $str)
    };
}

#[doc(hidden)]
#[cfg(not(feature = "enable-logger"))]
#[macro_export]
macro_rules! println_str {
    ($str:literal) => {
        $crate::print_cstr($crate::_export::cstr!(concat!($str, "\n")).into())
    };
    ($str:expr) => {
        $crate::println_fmt!("{}", $str)
    };
}

#[macro_export]
macro_rules! print {
    ($str:literal) => {
        $crate::print_str!($str)
    };
    ($str:expr) => {
        $crate::print_str!($str)
    };
    ($($tt:tt)*) => {
        $crate::print_fmt!($($tt)*)
    };
}

#[macro_export]
macro_rules! println {
    ($str:literal) => {
        $crate::println_str!($str)
    };
    ($str:expr) => {
        $crate::println_str!($str)
    };
    ($($tt:tt)*) => {
        $crate::println_fmt!($($tt)*)
    };
}
