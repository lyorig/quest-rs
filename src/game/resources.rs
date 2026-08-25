use std::time::Instant;

use halcyon::{
    Result, rect::PointF32, renderer::Renderer, resource::Resource, surface::Surface,
    window::Window,
};

use crate::{
    atlas::Atlas,
    chk,
    console::{self, CONSOLE_FONT, Cache, PREFIX_TEXT, Writer},
    font::store::{FontId, FontStore},
    util,
};

pub struct Resources<'t> {
    pub atlas: Atlas,

    pub renderer: Renderer,
    pub window: Window,

    pub fonts: FontStore<'t>,

    /// Caches the time at which the frame began, so that all calculations within a frame
    /// are consistent and only a single `rdtsc` (or similar) is performed.
    pub now: Instant,

    pub console: Option<console::Inner>,
    pub console_cache: Cache,
}

impl Resources<'_> {
    pub fn new<'t>(
        atlas: Atlas,
        renderer: Renderer,
        window: Window,
        fonts: FontStore<'t>,
    ) -> Resources<'t> {
        let console_cache = Cache::new(&fonts, renderer.as_ref());
        Resources {
            atlas,
            renderer,
            window,
            fonts,
            now: Instant::now(),
            console: None,
            console_cache,
        }
    }

    pub fn font_alloc(&mut self, i: FontId, text: &str) {
        self.fonts.alloc(i, text, &mut self.atlas);
    }

    pub fn font_free(&mut self, i: FontId, text: &str) {
        self.fonts.free(i, text);
    }

    pub fn font_draw(&self, id: FontId, text: &str, origin: &mut PointF32, glyph_size: PointF32) {
        self.fonts.draw(
            id,
            self.renderer.as_ref(),
            &self.atlas,
            text,
            origin,
            glyph_size,
        )
    }

    pub fn read_atlas_pixels(&self) -> Option<Result<Surface>> {
        self.atlas
            .texture
            .as_ref()
            .map(|t| util::read_pixels(self.renderer.as_ref(), t.as_ref()))
    }

    pub fn toggle_console(&mut self) {
        let wnd = self.window.as_ref();

        self.console = match self.console.take() {
            None => {
                chk!(halcyon::keyboard::text_input_start(wnd));

                let np = self.console_cache.next_placeholder();
                self.font_alloc(CONSOLE_FONT, np);
                self.font_alloc(CONSOLE_FONT, PREFIX_TEXT);

                let data = self.console_cache.writer.data();
                self.fonts.alloc(CONSOLE_FONT, data, &mut self.atlas);
                console::command::help_iter().for_each(|s| self.font_alloc(CONSOLE_FONT, s));

                Some(console::Inner::new(&mut self.console_cache))
            }

            Some(ac) => {
                chk!(halcyon::keyboard::text_input_stop(wnd));

                self.fonts
                    .free(CONSOLE_FONT, self.console_cache.writer.data());
                self.font_free(CONSOLE_FONT, PREFIX_TEXT);
                self.font_free(CONSOLE_FONT, self.console_cache.current_placeholder());
                self.font_free(CONSOLE_FONT, &ac.field.text);
                console::command::help_iter().for_each(|s| self.font_free(CONSOLE_FONT, s));

                None
            }
        };
    }

    pub fn writer(&mut self) -> &mut Writer {
        &mut self.console_cache.writer
    }
}
