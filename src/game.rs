use halcyon::{
    event::{Event, EventIter},
    renderer::{Renderer, RendererBuilder},
    subsystem::Video,
    window::{Window, WindowBuilder},
};

use crate::{atlas::Atlas, debugger::Debugger};

pub struct Game {
    window: Window,
    renderer: Renderer,
    running: bool,
    debug: Debugger,
    atlas: Atlas,
}

impl Game {
    /// Create a new game.
    pub fn new(vid: &Video) -> Self {
        let window = WindowBuilder::new()
            .size((640, 480))
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
            debug: Debugger::new(),
            atlas: Atlas::new(),
        }
    }

    /// Starts up the main loop.
    pub fn main_loop(&mut self) {
        // Pre-loop: print debug info.
        self.debug.print(&format!("Window ID {}", self.window.id()));

        while self.running {
            let _ = self.renderer.clear();

            for evt in EventIter::new() {
                match evt {
                    Event::Quit => self.running = false,
                    _ => (),
                }
            }

            self.atlas.pack(*self.renderer);

            let _ = self.renderer.present();
        }
    }
}
