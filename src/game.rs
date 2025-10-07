use std::time::Instant;

use halcyon::{
    color::Rgba,
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
        self.draw_gradient();
        let _ = self.renderer.present();

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
            for evt in EventIter::new() {
                match evt {
                    Event::Quit => self.running = false,
                    _ => (),
                }
            }
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

    fn draw_gradient(&self) {
        let prev_col = self.renderer.draw_color_f32();
        let mut size = self.renderer.output_size();

        let mut col = Rgba::rgb(1., 1., 1.);
        let mut step = -0.01;

        while size.x != 0 {
            self.renderer.set_draw_color_f32(col);
            let _ = self.renderer.draw_line(
                Point::new(size.x as _, 0.),
                Point::new(size.x as _, size.y as _),
            );

            col.rgb.r += step;
            if col.rgb.r <= 0. {
                col.rgb.r = 0.;
                step = 0.01;
            } else if col.rgb.r >= 1. {
                col.rgb.r = 1.;
                step = -0.01;
            }

            size.x -= 1;
        }

        self.renderer.set_draw_color_f32(prev_col);
    }
}
