use std::time::Instant;

use halcyon::{
    Result, gpu::Device, rect::PointF32, renderer::Renderer, resource::Resource, surface::Surface,
    window::Window,
};

use crate::{
    atlas::Atlas,
    font::{
        provider::*,
        store::{FontId, FontStore},
    },
    util,
};

type GP = RefcountGlyphMap;

pub struct Resources<'t> {
    pub atlas: Atlas,

    pub renderer: Renderer,
    pub window: Window,

    /// The GPU device backing [`Self::renderer`].
    ///
    /// It must outlive both the renderer and the window: the renderer
    /// releases the window from the device when it is dropped.
    pub device: Device,

    pub fonts: FontStore<'t, GP>,

    /// Caches the time at which the frame began, so that all calculations within a frame
    /// are consistent and only a single `rdtsc` (or similar) is performed.
    pub now: Instant,
}

impl<'t> Resources<'t> {
    pub fn new(
        atlas: Atlas,
        renderer: Renderer,
        window: Window,
        device: Device,
        fonts: FontStore<'t, GP>,
    ) -> Resources<'t> {
        Resources {
            atlas,
            renderer,
            window,
            device,
            fonts,
            now: Instant::now(),
        }
    }

    pub fn font_alloc(&mut self, i: FontId, text: &str) {
        self.fonts.alloc(i, text, &mut self.atlas);
    }

    pub fn font_free(&mut self, i: FontId, text: &str) {
        self.fonts.free(i, text);
    }

    pub fn font_draw(&self, id: FontId, text: &str, origin: &mut PointF32, glyph_size: PointF32) {
        self.fonts.draw(id, self, text, origin, glyph_size)
    }

    pub fn read_atlas_pixels(&self) -> Option<Result<Surface>> {
        self.atlas
            .texture
            .as_ref()
            .map(|t| util::read_pixels(self.renderer.as_ref(), t.as_ref()))
    }
}
