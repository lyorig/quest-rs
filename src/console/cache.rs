use std::rc::Rc;

const PLACEHOLDERS: [&str; 41] = [
    "[meow]",
    "[redacted]",
    "[your turn]",
    "[womp womp]",
    "[one big CVE]",
    "[kevin's heart]",
    "[lods of emone]",
    "[be not afraid]",
    "[see you again]",
    "[forget me not]",
    "[all is pretty]",
    "[stay vigilant]",
    "[sudo deez nuts]",
    "[openest source]",
    "[at your service]",
    "[with eye serene]",
    "[is anyone there?]",
    "[food for thought]",
    "[made with Halcyon]",
    "[49.0481N, 17.4838E]",
    "[are you satisfied?]",
    "[enter command here]",
    "[running out of time]",
    "[not actually random]",
    "[watch?v=lo5cG0FhWro]",
    "[not POSIX compliant]",
    "[start typing, please]",
    "[commands not included]",
    "[segfaulting since 2021]",
    "[waiting for user input]",
    "[non-euclidean interface]",
    "[who needs documentation]",
    "[no more parties in L.A.]",
    "[sudo pacman -S lyofetch]",
    "[no man page here, sorry]",
    "[licensed under the WTFPL]",
    "[streets and sodium lights]",
    "[quoth the raven, nevermore]",
    "[docker? I barely know 'er!]",
    "[rm -rf / --no-preserve-root]",
    "[MSVC is the real final boss]",
];

/// Not all data needs to be recreated every time the console is activated
/// (i.e. on every `ActiveConsole::new()`). This struct aims to achieve just
/// that, while also preventing double-mutable-borrow errors that would otherwise
/// occur if the calling `Console` passed itself as a parameter.
pub struct CachedData {
    /// The current index into `PLACEHOLDERS` used for
    /// generating, well, placeholders.
    pub placeholder_index: u8,

    /// The X coordinate of the input itself, equal to (placeholder names):
    /// `left_prefix_padding + prefix_length + right_prefix_padding`
    pub input_x_origin: f32,

    pub history: Vec<Rc<Box<str>>>,
}

impl CachedData {
    pub fn next_placeholder(&mut self) -> &'static str {
        self.advance_placeholder();
        self.current_placeholder()
    }

    pub fn current_placeholder(&self) -> &'static str {
        PLACEHOLDERS[self.placeholder_index as usize]
    }

    pub fn advance_placeholder(&mut self) {
        self.placeholder_index = (self.placeholder_index + 1) % PLACEHOLDERS.len() as u8;
    }
}
