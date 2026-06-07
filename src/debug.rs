use std::io::Write;
use std::{fmt::Arguments, io::stdout, time::Instant};

use crate::util::lazy_static::LazyStatic;

static EPOCH: LazyStatic<Instant> = LazyStatic::new();

/// Initialize the epoch used for printing debug message timestamps.
/// The epoch is initialized to [`Instant::now`].
pub fn init() {
    unsafe { EPOCH.init(Instant::now()) };
}

#[macro_export]
macro_rules! dprintln {
    ($($arg:tt)*) => {
        $crate::debug::print(format_args!($($arg)*))
    };
}

/// Prints a result of [`format_args`]. This function
/// is delegated to by [`dprintln`], and assumes you have already
/// called [`init`].
pub fn print(args: Arguments) {
    let mut lock = stdout().lock();
    let elapsed = unsafe { EPOCH.get() }.elapsed().as_secs_f32();
    _ = writeln!(lock, "[{elapsed:.3}] {args}");
}
