use halcyon::{
    defs::SdlResult, rect::PointF32, renderer::Renderer, surface::Surface, traits::Resource,
    window::Window,
};

use crate::{
    atlas::Atlas,
    font::store::{FontId, FontStore},
    util,
};

pub struct GameResources {
    pub atlas: Atlas,

    pub renderer: Renderer,
    pub window: Window,

    pub fonts: FontStore,
}

impl GameResources {
    pub fn new(atlas: Atlas, renderer: Renderer, window: Window, fonts: FontStore) -> Self {
        Self {
            atlas,
            renderer,
            window,
            fonts,
        }
    }

    pub fn font_alloc(&mut self, i: FontId, text: &str) {
        self.fonts.alloc(i, text, &mut self.atlas);
    }

    /// This function simply forwards to [`FontStore::free`],
    /// it's provided purely for completeness.
    pub fn font_free(&mut self, i: FontId, text: &str) {
        self.fonts.free(i, text);
    }

    pub fn font_gc(&mut self, i: FontId) {
        self.fonts.gc(i, &mut self.atlas);
    }

    pub fn font_gc_all(&mut self) {
        self.fonts.gc_all(&mut self.atlas);
    }

    pub fn font_draw(&self, id: FontId, text: &str, origin: &mut PointF32, glyph_size: PointF32) {
        self.fonts.draw(id, self, text, origin, glyph_size)
    }

    pub fn read_atlas_pixels(&self) -> Option<SdlResult<Surface>> {
        self.atlas
            .texture
            .as_ref()
            .map(|t| util::read_pixels(self.renderer.as_ref(), t.as_ref()))
    }
}
