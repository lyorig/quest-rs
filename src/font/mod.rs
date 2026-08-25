use std::ffi::CStr;

use halcyon::{
    Result,
    rect::PointF32,
    renderer::Renderer,
    resource::{Ref, Resource},
    ttf,
};

use crate::{atlas::Atlas, font::provider::GlyphProvider};

pub mod provider;
pub mod store;

/// A Halcyon Font whose glyphs are cached in an [`Atlas`].
pub struct Font<'t, GP: GlyphProvider> {
    pub font: ttf::Font<'t>,
    map: GP,
}

impl<'t, GP: GlyphProvider> Font<'t, GP> {
    pub fn new(
        ttf: &'t ttf::Context,
        font_path: &CStr,
        size: f32,
        atlas: &mut Atlas,
    ) -> Result<Self> {
        let font = ttf.open(font_path, size)?;

        assert!(
            font.is_mono(),
            "Font \"{}\" isn't fixed-width",
            font.family()
        );

        let map = GP::new(atlas, font.as_ref());

        Ok(Self { font, map })
    }

    /// Calls [`GlyphProvider::retain`] on the contained map; see its
    /// documentation for more information.
    pub fn alloc(&mut self, text: &str, atlas: &mut Atlas) {
        self.map.retain(atlas, self.font.as_ref(), text);
    }

    /// Calls [`GlyphProvider::release`] on the contained map; see its
    /// documentation for more information.
    pub fn free(&mut self, text: &str) {
        self.map.release(text);
    }

    pub fn draw(
        &self,
        text: &str,
        rnd: Ref<Renderer>,
        atlas: &Atlas,
        origin: &mut PointF32,
        glyph_size: PointF32,
    ) {
        self.map.draw(text, rnd, atlas, origin, glyph_size);
    }
}
