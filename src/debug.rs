use std::cell::OnceCell;
use std::io::Write;
use std::{fmt::Arguments, io::stdout, time::Instant};

struct Wrapper {
    inner: OnceCell<Instant>,
}

impl Wrapper {
    const fn new() -> Self {
        Self {
            inner: OnceCell::new(),
        }
    }

    fn elapsed(&self) -> f32 {
        // SAFETY: Debug calls only happen after the epoch has been initialized.
        let inst = unsafe { self.inner.get().unwrap_unchecked() };
        inst.elapsed().as_secs_f32()
    }
}

// SAFETY: We only use debug facilities on the main thread.
unsafe impl Sync for Wrapper {}

static EPOCH: Wrapper = Wrapper::new();

/// Initialize the epoch used for printing debug message timestamps.
/// The epoch is initialized to [`Instant::now`].
pub fn init_epoch() {
    _ = EPOCH.inner.set(Instant::now());
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
    let mut lock = stdout().lock();
    _ = writeln!(lock, "[{:.3}] {}", EPOCH.elapsed(), args);
}
