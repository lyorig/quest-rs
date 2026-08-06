#[macro_export]
macro_rules! dprintln {
    ($($arg:tt)*) => {
        halcyon::log!($($arg)*)
    };
}
