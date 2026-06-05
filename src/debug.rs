use std::io::Write;
use std::sync::OnceLock;
use std::{fmt::Arguments, io::stdout, time::Instant};

static EPOCH: OnceLock<Instant> = OnceLock::new();

/// Initialize the epoch used for printing debug message timestamps.
/// The epoch is initialized to [`Instant::now`].
pub fn init_epoch() {
    _ = EPOCH.set(Instant::now());
}

#[macro_export]
macro_rules! dprint {
    ($($arg:tt)*) => {
        $crate::debug::print(format_args!($($arg)*))
    };
}

/// Prints a result of [`format_args`]. If the debug epoch hasn't
/// been initialized yet, it's set to [`Instant::now`]. This function
/// is delegated to by [`dprint`].
pub fn print(args: Arguments) {
    // PERF: Replace with EPOCH.get().unwrap_unchecked().
    let elapsed = EPOCH.get_or_init(Instant::now).elapsed().as_secs_f32();
    let mut lock = stdout().lock();

    _ = writeln!(lock, "[{:.3}] {}", elapsed, args);
}
