use std::time::Instant;

use halcyon::{
    event::{Event, EventIter},
    rect::Point,
    renderer::{Renderer, RendererBuilder},
    subsystem::Video,
    window::{Window, WindowBuilder},
};

use crate::atlas::Atlas;

pub struct Game {
    window: Window,
    renderer: Renderer,
    running: bool,
    atlas: Atlas,
    epoch: Instant,
}

impl Game {
    /// Create a new game.
    pub fn new(vid: &Video) -> Self {
        let window = WindowBuilder::new()
            .size(Point::new(640, 480))
            .title(c"HalodaQuest [Euclid]")
            .build(vid)
            .expect("Window creation failed");

        let renderer = RendererBuilder::new(&window)
            .vsync(1)
            .build()
            .expect("Renderer creation failed");

        Self {
            window,
            renderer,
            running: true,
            atlas: Atlas::new(),
            epoch: Instant::now(),
        }
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

            for evt in EventIter::new() {
                match evt {
                    Event::Quit => self.running = false,
                    _ => (),
                }
            }

            self.atlas.pack(&self.renderer);

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
