use std::ffi::CStr;

use halcyon::{
    rect::PointF32,
    renderer::RendererRef,
    ttf::{Font as HalFont, Text, TtfContext},
};

use crate::{atlas::Atlas, font::glyph_map::GlyphMap};

pub mod glyph_map;
pub mod store;

pub struct Font<'a> {
    font: HalFont<'a>,
    pub glyph_size: PointF32,
    map: GlyphMap,
}

/// A wrapper around a Halcyon font with an added glyph map
/// and convenience methods.
impl Font<'_> {
    pub fn new<'a>(ttf: &'a TtfContext, font_path: &CStr, size: f32) -> Font<'a> {
        let font = HalFont::new(ttf, font_path, size).expect("Cannot open font");
        assert!(
            font.is_mono(),
            "Font \"{}\" isn't fixed-width",
            font.family()
        );

        let glyph_size = Text::new(&font, "X").unwrap().size().into();

        Font {
            font,
            glyph_size,
            map: GlyphMap::new(),
        }
    }

    /// Calls [`GlyphMap::retain()`] on the contained map; see its
    /// documentation for more information.
    pub fn alloc(&mut self, text: &str, atlas: &mut Atlas) {
        self.map.retain(atlas, *self.font, text);
    }

    /// Calls [`GlyphMap::release()`] on the contained map; see its
    /// documentation for more information.
    pub fn free(&mut self, text: &str) {
        self.map.release(text);
    }

    /// Calls [`GlyphMap::gc()`] on the contained map; see its
    /// documentation for more information.
    pub fn gc(&mut self, atlas: &mut Atlas) {
        self.map.gc(atlas);
    }

    pub fn draw(
        &self,
        text: &str,
        atlas: &Atlas,
        renderer: RendererRef,
        origin: &mut PointF32,
        glyph_size: PointF32,
    ) {
        self.map.draw(text, atlas, renderer, origin, glyph_size);
    }
}
