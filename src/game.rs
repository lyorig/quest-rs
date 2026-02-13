use std::time::{Duration, Instant};

use halcyon::{
    color::Rgba,
    defs::SdlResult,
    event::{Event, EventIter},
    guard::DrawColorGuard,
    rect::{Point, PointF32, Rect},
    renderer::{Renderer, RendererBuilder},
    resource_loader::ResourceLoader,
    subsystem::Video,
    traits::BlendMode,
    ttf::TtfContext,
    util::c_ptr_to_str,
    window::{Window, WindowBuilder},
};

use sdl3_sys::{blendmode::SDL_BLENDMODE_BLEND, keycode::*};

use crate::{
    atlas::Atlas,
    console::{Console, state::ConsoleState},
    font::store::{FontId, Fonts},
};

pub struct GameData<'a> {
    pub running: bool,

    pub atlas: Atlas,

    pub renderer: Renderer,
    pub window: Window,

    pub fonts: Fonts<'a>,
}

impl GameData<'_> {
    pub fn draw_atlas(&self) {
        if let Some(at) = self.atlas.texture.as_ref() {
            let origin = Point::new(300.0, 300.0);
            let sz = Rect::new(origin, at.size());

            let _dcl = DrawColorGuard::new(&self.renderer, Rgba::BLACK);

            let _ = self.renderer.draw_rect(sz);
            let _ = self.renderer.draw(at, None, Some(&sz));

            self.atlas.debug_draw(*self.renderer, origin);
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
        self.fonts
            .draw(id, &self.atlas, *self.renderer, text, origin, glyph_size)
    }
}

pub struct Game<'a> {
    pub data: GameData<'a>,
    console: Console,
}

impl Game<'_> {
    /// Create a new game.
    ///
    /// SAFETY: Ensure a valid [`TtfContext`] exists for the
    /// lifetime of the returned [`Game`].
    pub unsafe fn new<'a>(vid: &Video, ttf: &'a TtfContext) -> SdlResult<Game<'a>> {
        let window = WindowBuilder::new()
            .size(Point::new(1280, 720))
            .position(Point::new(Window::POS_CENTERED, Window::POS_CENTERED))
            .title(c"HalodaQuest [Euclid]")
            .build(vid)?;

        let mut renderer = RendererBuilder::new(&window);
        if !std::env::args().any(|x| x == "--no-vsync") {
            renderer.vsync(1);
        }

        let renderer = renderer.build()?;
        renderer.set_blend_mode(SDL_BLENDMODE_BLEND);

        let res_ldr = ResourceLoader::new();

        let data = GameData {
            running: true,
            atlas: Atlas::new(),
            renderer,
            window,
            fonts: Fonts::new(ttf, res_ldr),
        };

        let console = Console::new(*data.renderer)?;

        Ok(Game { data, console })
    }

    /// Starts up the main loop.
    pub fn main_loop(&mut self) {
        let mut delta = Instant::now();

        // I could probably just use a named loop and break it in case
        // of a quit event, but there are two issues:
        //
        // 1) The Game class cannot easily be told to quit from other classes.
        // 2) There are potentially important things running in the loop
        // after events are polled, so breaking in the middle of polling events
        // could cause some issues.
        //
        // In any case, it's literally one extra byte in exchange for a whole
        // lot of extra flexibility, so I don't particularly mind implementing
        // things this way.
        while self.data.running {
            let _col = DrawColorGuard::new(&self.data.renderer, Rgba::rgb(0.0, 0.0, 0.75));
            let _ = self.data.renderer.clear();

            // --- Processing ---
            self.process_events();

            // --- Updating ---
            self.update_delta(delta.elapsed());
            delta = Instant::now();

            self.data.atlas.pack(*self.data.renderer);

            // --- Drawing ---
            self.console.draw(&self.data);
            self.data.draw_atlas();

            let _ = self.data.renderer.present();
        }
    }

    fn process_events(&mut self) {
        for evt in EventIter::new() {
            match evt {
                Event::Quit => self.data.running = false,
                Event::KeyDown(k) => match k.key {
                    SDLK_F1 => self.console.switch(&mut self.data),
                    SDLK_RETURN => self.process_command(),
                    other => self.console.process_key(&mut self.data, other),
                },
                Event::TextInput(ti) => {
                    self.console
                        .process_str(&mut self.data, unsafe { c_ptr_to_str(ti.text) });
                }
                _ => (),
            }
        }
    }

    fn process_command(&mut self) {
        if let ConsoleState::Enabled(ac) = &mut self.console.state {
            ac.process_command(&mut self.console.data, &mut self.data);
        }
    }

    fn update_delta(&mut self, elapsed: Duration) {
        self.console.update_delta(elapsed);
    }
}
