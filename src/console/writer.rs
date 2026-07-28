/// Essentially a wrapper around a [`String`].
/// Used for commands to output text.
/// Also kind of contains the console history.
pub struct Writer {
    data: String,
    last_added: u32,
}

impl Writer {
    pub fn new() -> Self {
        Self {
            data: String::new(),
            last_added: 0,
        }
    }

    pub fn write_char(&mut self, c: char) {
        self.data.push(c);
    }

    pub fn write(&mut self, data: &str) {
        self.data.push_str(data);
    }

    pub fn writeln(&mut self, data: &str) {
        self.write(data);
        self.write_char('\n');
    }

    pub fn write_command(&mut self, cmd: &str) {
        self.write_char('\0');
        self.writeln(cmd);
    }

    pub fn lines(&self) -> impl Iterator<Item = &'_ str> {
        self.data.split('\n')
    }

    pub fn data(&self) -> &str {
        &self.data
    }

    /// Returns a string slice containing all text that has been
    /// added since the last call to this method.
    pub fn added_since_last_check(&mut self) -> &str {
        if self.missing_newline() {
            self.write("%\n");
        }

        let slice = &self.data[self.last_added as usize..];
        self.last_added = (self.data.len() + 1) as _; // Compensate for \n.
        slice
    }

    fn missing_newline(&self) -> bool {
        self.data.bytes().last().is_some_and(|b| b != b'\n')
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.last_added = 0;
    }
}

impl std::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.write(s);
        Ok(())
    }

    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.write_char(c);
        Ok(())
    }
}
