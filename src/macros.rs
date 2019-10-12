#[macro_export]
macro_rules! print {
    ($str:expr) => {
        $crate::print_cstr($crate::_export::cstr!($str).into())
    };
    ($($tt:tt)*) => {
        {
            use $crate::_export::core::fmt::Write;
            // failures aren't interesting to us
            let _ = $crate::_export::core::write!($crate::CharPrinter, $($tt)*);
        }
    };
}

#[macro_export]
macro_rules! println {
    ($str:expr) => {
        $crate::print_cstr($crate::_export::cstr!(concat!($str, "\n")).into())
    };
    ($($tt:tt)*) => {
        {
            use $crate::_export::core::fmt::Write;
            let _ = $crate::_export::core::writeln!($crate::CharPrinter, $($tt)*);
        }
    };
}
