use std::cell::UnsafeCell;
use std::io::Write;
use std::mem::MaybeUninit;
use std::{fmt::Arguments, io::stdout, time::Instant};

struct Wrapper {
    inner: UnsafeCell<MaybeUninit<Instant>>,
}

impl Wrapper {
    const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    fn elapsed(&self) -> f32 {
        // SAFETY: Debug calls only happen after the epoch has been initialized.
        let ptr = unsafe { *self.inner.get() };
        let inst = unsafe { ptr.assume_init() };
        inst.elapsed().as_secs_f32()
    }

    fn init_epoch(&self) {
        let r = unsafe { self.inner.get().as_mut_unchecked() };
        r.write(Instant::now());
    }
}

// SAFETY: We only use debug facilities on the main thread.
unsafe impl Sync for Wrapper {}

static EPOCH: Wrapper = Wrapper::new();

/// Initialize the epoch used for printing debug message timestamps.
/// The epoch is initialized to [`Instant::now`].
pub fn init_epoch() {
    EPOCH.init_epoch();
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
