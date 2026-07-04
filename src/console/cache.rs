use halcyon::rect::{PointF32, RectF32};

use crate::{
    console::{PREFIX_TEXT, active::TEXT_OFFSET, writer::ConsoleWriter},
    game::resources::Resources,
};

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

/// Not all data needs to/should be recreated every time the console is activated
/// (i.e. on every `ActiveConsole::new()`). This struct aims to achieve just
/// that, while also preventing double-mutable-borrow errors that would otherwise
/// occur if the calling `Console` passed itself as a parameter.
pub struct CachedData {
    /// The current index into [`PLACEHOLDERS`] used for
    /// generating, well, placeholders.
    pub placeholder_index: u8,

    /// The X coordinate of the input itself.
    pub input_x_origin: f32,

    /// The desired size of a console glyph.
    pub glyph_size: PointF32,

    pub writer: ConsoleWriter,
}

impl CachedData {
    pub fn new() -> Self {
        let glyph_size = PointF32::new(16.0, 32.0);

        Self {
            placeholder_index: 0,
            input_x_origin: TEXT_OFFSET.x + glyph_size.x * (PREFIX_TEXT.len() + 1) as f32,
            glyph_size,
            writer: ConsoleWriter::new(),
        }
    }

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

    pub fn scroll_bar(&self, res: &Resources) -> RectF32 {
        let lines = self.writer.lines().count();
        let height = self.glyph_size.y * lines as f32;

        let sz = res.renderer.output_size();
        let wndy = sz.y as f32;

        let ratio = wndy / height;
        let bar_height = wndy * ratio;

        RectF32::xywh(sz.x as f32 - 10., 0., 10., bar_height)
    }
}
