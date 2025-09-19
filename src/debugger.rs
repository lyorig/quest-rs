use std::io::Write;
use std::{io::stdout, time::Instant};

pub struct Debugger {
    epoch: Instant,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }

    /// Print a string to `stdout`.
    pub fn print(&self, args: &str) {
        let mut lock = stdout().lock();
        let _ = write!(lock, "[{:.3}] ", self.epoch.elapsed().as_secs_f32());
        let _ = lock.write(args.to_string().as_bytes());
        let _ = lock.write(b"\n");
    }
}
