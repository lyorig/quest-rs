use std::{iter::Skip, str::Split};

/// Essentially a wrapper around a `String`.
/// Used for commands to output text.
pub struct ConsoleWriter {
    data: String,
}

impl ConsoleWriter {
    pub fn new() -> Self {
        Self {
            data: String::new(),
        }
    }

    pub fn write(&mut self, data: &str) {
        self.data.push('\n');
        self.data.push_str(data);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn lines<'a>(&'a self) -> Skip<Split<'a, char>> {
        self.data.split('\n').skip(1)
    }

    pub fn data(&self) -> &str {
        self.data.as_str()
    }
}
