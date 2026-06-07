use halcyon::{
    color::Rgba,
    rect::{PointF32, RectF32},
    renderer::Renderer,
    traits::Resource,
    window::Window,
};

use crate::{
    atlas::Atlas,
    font::store::{FontId, FontStore},
};

pub struct GameResources {
    pub running: bool,

    pub atlas: Atlas,

    pub renderer: Renderer,
    pub window: Window,

    pub fonts: FontStore,
}

impl GameResources {
    pub fn draw_atlas(&self) {
        if let Some(at) = self.atlas.texture.as_ref() {
            let origin = PointF32::new(300.0, 300.0);
            let sz = RectF32::new(origin, at.size());

            let old_col = self.renderer.xchg_draw_color_f32(Rgba::BLACK);

            _ = self.renderer.draw_rect(sz);
            _ = self.renderer.draw(at.as_ref(), None, Some(&sz));

            self.atlas.debug_draw(self.renderer.as_ref(), origin);

            self.renderer.set_draw_color_f32(old_col);
        }
    }

    pub fn font_alloc(&mut self, i: FontId, text: &str) {
        self.fonts.alloc(i, text, &mut self.atlas);
    }

    /// This function simply forwards to [`Fonts::free()`],
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
}
