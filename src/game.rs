use halcyon::{traits::BlendMode as _, ttf};
use std::time::Instant;

use halcyon::{
    Result,
    color::Rgba,
    event::{Event, EventIter},
    pixels::BlendMode,
    properties::Properties,
    rect::{Point, PointF32},
    renderer::Renderer,
    resource::Resource,
    surface::Surface,
    util::c_ptr_to_str,
    window::Window,
};

use sdl3_sys::keycode::*;

use crate::{
    atlas::Atlas,
    chk,
    console::{self, CONSOLE_FONT, PREFIX_TEXT},
    font::store::{FontId, FontStore},
    util::{self, resource_loader::ResourceLoader},
};

pub struct Game<'t> {
    pub atlas: Atlas,

    pub renderer: Renderer,
    pub window: Window,

    pub fonts: FontStore<'t>,

    /// Caches the time at which the frame began, so that all calculations within a frame
    /// are consistent and only a single `rdtsc` (or similar) is performed.
    pub now: Instant,

    pub console: Option<console::Inner>,
    pub console_cache: console::Cache,
}

impl<'t> Game<'t> {
    pub fn new(ttf: &'t ttf::Context) -> Result<Self> {
        let props = Properties::global()?;

        let window = Window::builder(props)
            .title(c"HalodaQuest")
            .size(Point::new(1280, 720))
            .position(Point::new(Window::POS_CENTERED, Window::POS_CENTERED))
            .resizable(true)
            .build_cleanup()?;

        let renderer = Renderer::builder(props)
            .window(window.as_ref())
            .vsync(1)
            .build_cleanup()?;

        renderer.set_blend_mode(BlendMode::Blend);

        let res_ldr = ResourceLoader::from_pref()?;
        let mut atlas = Atlas::new();
        let fonts = FontStore::new(ttf, res_ldr, &mut atlas)?;
        let console_cache = console::Cache::new(&fonts, renderer.as_ref());

        Ok(Self {
            atlas,
            renderer,
            window,
            fonts,
            now: Instant::now(),
            console: None,
            console_cache,
        })
    }

    pub fn main_loop(&mut self) {
        loop {
            let old = self.renderer.draw_color_f32();
            self.renderer.set_draw_color_f32(Rgba::BLACK);

            chk!(self.renderer.clear());

            // --- Processing ---
            if !self.process_events() {
                break;
            }

            // --- Updating ---
            let now = Instant::now();
            let dt = now.duration_since(self.now);
            self.now = now;

            if let Some(ref mut c) = self.console {
                c.update_delta(dt);
            }

            self.atlas.pack(self.renderer.as_ref());

            if let Some(ref mut c) = self.console {
                c.draw(
                    &mut self.console_cache,
                    self.renderer.as_ref(),
                    &mut self.fonts,
                    &mut self.atlas,
                );
            }

            chk!(self.renderer.present());

            self.renderer.set_draw_color_f32(old);
        }
    }

    fn process_events(&mut self) -> bool {
        for evt in EventIter::new() {
            match evt {
                Event::Quit => return false,
                Event::KeyDown(k) => match k.key {
                    SDLK_F1 => self.toggle_console(),
                    SDLK_RETURN => self.process_command(),
                    other => {
                        if let Some(ref mut c) = self.console {
                            c.process_key(
                                &mut self.console_cache,
                                &mut self.fonts,
                                &mut self.atlas,
                                other,
                            );
                        }
                    }
                },
                Event::MouseWheelMotion(m) => {
                    if let Some(ref mut c) = self.console {
                        c.process_mouse(self.renderer.as_ref(), &mut self.console_cache, m);
                    }
                }
                Event::TextInput(ti) => {
                    let text = unsafe { c_ptr_to_str(ti.text) };
                    if let Some(ref mut c) = self.console {
                        c.process_str(
                            &mut self.console_cache,
                            &mut self.fonts,
                            &mut self.atlas,
                            text,
                        );
                    }
                }
                _ => (),
            }
        }

        true
    }

    fn process_command(&mut self) {
        self.console = self.console.take().map(|mut c| {
            c.process_command(self);
            c
        });
    }

    pub fn quit() {
        chk!(Event::Quit.push());
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

    pub fn writer(&mut self) -> &mut console::Writer {
        &mut self.console_cache.writer
    }
}
