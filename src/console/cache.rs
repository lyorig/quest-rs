use halcyon::rect::{PointF32, RectF32};

use crate::{
    console::{CONSOLE_FONT, PREFIX_TEXT, active::TEXT_OFFSET, writer::ConsoleWriter},
    dprintln,
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

    pub scroll_bar: RectF32,
    pub line: u32,

    pub max_lines: i32,
}

impl CachedData {
    pub fn new(res: &Resources) -> Self {
        let glyph_size = PointF32::new(16.0, 32.0);
        let wnd_y = res.renderer.output_size().y;
        let amnt = wnd_y / glyph_size.y as i32;

        dprintln!("Console max lines = {amnt}");

        Self {
            placeholder_index: 0,
            input_x_origin: TEXT_OFFSET.x + glyph_size.x * (PREFIX_TEXT.len() + 1) as f32,
            glyph_size,
            writer: ConsoleWriter::new(),
            scroll_bar: RectF32::ZEROED,
            line: 0,
            max_lines: amnt,
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

    pub fn update_scroll_bar(&mut self, res: &Resources) {
        let lines = self.writer.lines().count();
        let height = self.glyph_size.y * lines as f32;

        let sz = res.renderer.output_size();
        let wndy = sz.y as f32;

        let ratio = wndy / height;
        let bar_height = wndy * ratio;

        let x_ratio = self.line as f32 / self.bottom() as f32;

        self.scroll_bar = RectF32::xywh(
            sz.x as f32 - 10.,
            (wndy - bar_height) * x_ratio,
            10.,
            bar_height,
        );
    }

    pub fn clear(&mut self, res: &mut Resources) {
        res.font_free(CONSOLE_FONT, self.writer.data());
        self.writer.clear();
        self.line = 0;
    }

    pub fn bottom(&self) -> usize {
        self.writer
            .lines()
            .count()
            .saturating_sub_signed(self.max_lines as _)
            + 1
    }
}
