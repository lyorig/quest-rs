use std::io::Write;
use std::{fmt::Arguments, io::stdout, time::Instant};

#[macro_export]
macro_rules! dprint {
    ($epoch:expr, $fmt:literal, $($t:tt)*) => {
        crate::debug::print($epoch, format_args!(concat!($fmt, "\n"), $($t)*))
    };
}

pub fn print(epoch: Instant, args: Arguments) {
    let mut lock = stdout().lock();
    let _ = write!(lock, "[{:.3}] ", epoch.elapsed().as_secs_f32());
    let _ = write!(lock, "{}", args.to_string());
}
