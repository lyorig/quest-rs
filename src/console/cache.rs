use halcyon::{
    rect::{PointF32, PointI32, RectF32},
    renderer::Renderer,
    resource::Ref,
};

use crate::{
    console::{CONSOLE_FONT, PREFIX_TEXT, inner::TEXT_OFFSET, writer::Writer},
    font::store::FontStore,
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
/// (i.e. on every [`super::Inner::new`]). This struct aims to achieve just
/// that, while also preventing double-mutable-borrow errors that would otherwise
/// occur if the calling [`super::Console`] passed itself as a parameter.
pub struct Cache {
    /// The current index into [`PLACEHOLDERS`] used for
    /// generating, well, placeholders.
    pub placeholder_index: u8,

    /// The X coordinate of the input itself.
    pub input_x_origin: f32,

    /// The desired size of a console glyph.
    pub glyph_size: PointF32,

    pub writer: Writer,

    pub scroll_bar: RectF32,
    pub line: u32,

    pub max_lines: i32,
}

impl Cache {
    pub fn new(fonts: &FontStore, rnd: Ref<Renderer>) -> Self {
        let glyph_size = fonts.glyph_size(CONSOLE_FONT).to_f32();

        let mut ret = Self {
            placeholder_index: 0,
            input_x_origin: TEXT_OFFSET.x + glyph_size.x * (PREFIX_TEXT.len() + 1) as f32,
            glyph_size,
            writer: Writer::new(),
            scroll_bar: RectF32::ZEROED,
            line: 0,
            max_lines: 0,
        };

        ret.resize(rnd.output_size(), rnd);

        ret
    }

    pub fn resize(&mut self, size: PointI32, rnd: Ref<Renderer>) {
        let amnt = size.y / self.glyph_size.y as i32;

        self.max_lines = amnt - 1;
        self.clamp_line(rnd);
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

    pub fn update_scroll_bar(&mut self, rnd: Ref<Renderer>) {
        let lines = self.writer.lines().count();
        let height = self.glyph_size.y * lines as f32;

        let sz = rnd.output_size();
        let wndy = sz.y as f32;

        let ratio = wndy / height;
        let bar_height = wndy * ratio;

        let x_ratio = self.line as f32 / self.total_lines() as f32;

        self.scroll_bar = RectF32::xywh(
            sz.x as f32 - 10.,
            (wndy - bar_height) * x_ratio,
            10.,
            bar_height,
        );
    }

    pub fn clear(&mut self, fonts: &mut FontStore) {
        fonts.free(CONSOLE_FONT, self.writer.data());
        self.writer.clear();
        self.line = 0;
    }

    pub fn total_lines(&self) -> usize {
        self.writer
            .lines()
            .count()
            .saturating_sub_signed(self.max_lines as _)
            + 1
    }

    pub fn clamp_line(&mut self, rnd: Ref<Renderer>) {
        self.line = self.line.clamp(0, self.total_lines() as _);
        self.update_scroll_bar(rnd);
    }
}
