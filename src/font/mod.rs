use std::ffi::CStr;

use halcyon::{rect::PointF32, traits::Resource, ttf};

use crate::{atlas::Atlas, font::glyph_map::GlyphMap, game::resources::Resources};

pub mod glyph_map;
pub mod store;

/// A Halcyon Font whose glyphs are cached in an [`Atlas`].
pub struct Font<'t> {
    font: ttf::Font<'t>,
    map: GlyphMap,
}

impl Font<'_> {
    pub fn new<'t>(ttf: &'t ttf::Context, font_path: &CStr, size: f32) -> Font<'t> {
        let font = ttf.open(font_path, size).expect("Cannot open font");

        assert!(
            font.is_mono(),
            "Font \"{}\" isn't fixed-width",
            font.family()
        );

        Font {
            font,
            map: GlyphMap::new(),
        }
    }

    /// Calls [`GlyphMap::retain()`] on the contained map; see its
    /// documentation for more information.
    pub fn alloc(&mut self, text: &str, atlas: &mut Atlas) {
        self.map.retain(atlas, self.font.as_ref(), text);
    }

    /// Calls [`GlyphMap::release()`] on the contained map; see its
    /// documentation for more information.
    pub fn free(&mut self, text: &str) {
        self.map.release(text);
    }

    /// Calls [`GlyphMap::gc()`] on the contained map; see its
    /// documentation for more information.
    pub fn gc(&mut self, atlas: &mut Atlas) -> usize {
        self.map.gc(atlas)
    }

    pub fn draw(&self, text: &str, res: &Resources, origin: &mut PointF32, glyph_size: PointF32) {
        self.map.draw(text, res, origin, glyph_size);
    }
}
