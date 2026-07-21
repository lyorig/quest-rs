/// Essentially a wrapper around a [`String`].
/// Used for commands to output text.
/// Also kind of contains the console history.
pub struct ConsoleWriter {
    data: String,
    last_added: u32,
}

impl ConsoleWriter {
    pub fn new() -> Self {
        Self {
            data: String::new(),
            last_added: 1,
        }
    }

    pub fn write(&mut self, data: &str) {
        self.data.push_str(data);
    }

    pub fn writeln(&mut self, data: &str) {
        self.data.push('\n');
        self.write(data);
    }

    pub fn write_command(&mut self, cmd: &str) {
        self.data.push('\n');
        self.data.push('\0');
        self.write(cmd);
    }

    pub fn lines(&self) -> impl Iterator<Item = &'_ str> {
        self.data.split('\n').skip(1)
    }

    pub fn data(&self) -> &str {
        &self.data
    }

    /// Returns a string slice containing all text that has been
    /// added since the last call to this method.
    pub fn added_since_last_check(&mut self) -> &str {
        if self.last_added >= self.data.len() as _ {
            // NOTE: This fixes a situation when the last command provided no output.
            // The `last_added` field is offset by 1 to skip over newlines,
            // which isn't present in this case.
            ""
        } else {
            let slice = &self.data[self.last_added as usize..];
            self.last_added = (self.data.len() + 1) as _; // Compensate for \n.

            slice
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.last_added = 1;
    }
}
