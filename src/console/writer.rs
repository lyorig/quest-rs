use std::{iter::Skip, str::Split};

/// Essentially a wrapper around a `String`.
/// Used for commands to output text.
pub struct ConsoleWriter {
    data: String,
    last_added: usize,
}

impl ConsoleWriter {
    pub fn new() -> Self {
        Self {
            data: String::new(),
            last_added: 1,
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
        &self.data
    }

    /// Returns a string slice containing all text that has been
    /// added since the last call to this method.
    pub fn added_since_last_check(&mut self) -> &str {
        if self.last_added >= self.data.len() {
            // NOTE: This fixes a situation when the last command provided no output.
            // The `last_added` field is offset by 1 to skip over newlines,
            // which isn't present in this case.
            ""
        } else {
            let slice = &self.data[self.last_added..];
            self.last_added = self.data.len() + 1; // Compensate for \n.

            slice
        }
    }
}
