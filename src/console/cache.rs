use std::rc::Rc;

use halcyon::{rect::PointF32, ttf::Font};

use crate::glyph_map::GlyphMap;

/// Not all data needs to be recreated every time the console is activated
/// (i.e. on every `ActiveConsole::new()`). This struct aims to achieve just
/// that, while also preventing double-mutable-borrow errors that would otherwise
/// occur if the calling `Console` passed itself as a parameter.
pub struct CachedData<'a> {
    /// The current index into `PLACEHOLDERS` used for
    /// generating, well, placeholders.
    pub placeholder_index: u8,

    /// The font that is used to render text.
    pub font: Font<'a>,

    /// The X coordinate of the input itself, equal to (placeholder names):
    /// `left_prefix_padding + prefix_length + right_prefix_padding`
    pub input_x_origin: f32,

    /// This is cached because the size component is the size of
    /// one glyph, and since `Self::font` is cached as well, we don't
    /// need to re-calculate it on every console activation.
    pub glyph_size: PointF32,
    pub glyph_map: GlyphMap,

    pub history: Vec<Rc<Box<str>>>,
}
