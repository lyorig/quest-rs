use std::time::Instant;

use halcyon::{
    color::Rgba,
    defs::SdlResult,
    event::{Event, EventIter},
    guard::DrawColorGuard,
    rect::{Point, Rect},
    renderer::{Renderer, RendererBuilder},
    resource_loader::ResourceLoader,
    subsystem::Video,
    ttf::TtfContext,
    window::{Window, WindowBuilder},
};

use sdl3_sys::blendmode::SDL_BLENDMODE_BLEND;

use crate::{atlas::Atlas, console::Console};

pub struct Game {
    pub running: bool,
    pub epoch: Instant,
    pub res_ldr: ResourceLoader,

    pub atlas: Atlas,
    console: Console,

    pub renderer: Renderer,
    pub window: Window,

    pub ttf: TtfContext,

    pub events: Vec<Event>,
}

impl Game {
    /// Create a new game.
    pub fn new(vid: &Video) -> SdlResult<Self> {
        let ttf = TtfContext::new().expect("Should be able to initialize TTF");

        let window = WindowBuilder::new()
            .size(Point::new(1280, 720))
            .position(Point::new(Window::POS_CENTERED, Window::POS_CENTERED))
            .title(c"HalodaQuest [Euclid]")
            .build(vid)?;

        let renderer = RendererBuilder::new(&window).vsync(1).build()?;

        renderer.set_blend_mode(SDL_BLENDMODE_BLEND);
        renderer.set_draw_color_f32(Rgba::rgb(0.0, 0.0, 0.75));
        let _ = renderer.clear();

        let epoch = Instant::now();
        let res_ldr = ResourceLoader::new();
        let console = Console::new(&renderer, &ttf, epoch, res_ldr)?;

        Ok(Self {
            running: true,
            epoch: Instant::now(),
            res_ldr,
            atlas: Atlas::new(),
            console,
            renderer,
            window,
            ttf,
            events: Vec::new(),
        })
    }

    /// Starts up the main loop.
    pub fn main_loop(&mut self) {
        #[cfg(debug_assertions)]
        self.print_debug_data();

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
        while self.running {
            let _ = self.renderer.clear();
            self.events = EventIter::new().collect();

            // The game itself is interested in certain events.
            for evt in &self.events {
                match evt {
                    Event::Quit => self.running = false,
                    _ => (),
                }
            }

            self.console
                .process_events(&self.events, &mut self.atlas, &self.window);

            self.atlas.pack(&self.renderer);
            self.console.try_draw(&self.renderer, &mut self.atlas);

            if let Some(at) = self.atlas.texture.as_ref() {
                let sz = Rect::new(Point::new(300.0, 300.0), at.size());
                let _col = DrawColorGuard::new(&self.renderer, Rgba::BLACK);

                let _ = self.renderer.draw_rect(sz);
                let _ = self.renderer.draw(at, None, Some(&sz));
            }

            let _ = self.renderer.present();
        }
    }

    #[cfg(debug_assertions)]
    fn print_debug_data(&self) {
        use crate::dprint;
        use halcyon::context::Context;

        let e = self.epoch;

        dprint!(e, "Running on {}", Context::platform());
        dprint!(e, "Window ID {}", self.window.id());
        dprint!(
            e,
            "Rendering via \"{}\" ({} available in total)",
            self.renderer.name(),
            Renderer::num_drivers()
        );
    }
}
