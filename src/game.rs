use std::time::{Duration, Instant};

use halcyon::{
    color::Rgba,
    defs::SdlResult,
    event::{Event, EventIter},
    guard::DrawColorGuard,
    rect::{Point, Rect},
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
    console::{Console, ConsoleState},
};

pub struct Game {
    pub running: bool,

    pub atlas: Atlas,
    console: Console,

    pub renderer: Renderer,
    pub window: Window,

    pub _ttf: TtfContext,
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

        let mut renderer = RendererBuilder::new(&window);
        if !std::env::args().any(|x| x == "--no-vsync") {
            renderer.vsync(1);
        }

        let renderer = renderer.build()?;

        renderer.set_blend_mode(SDL_BLENDMODE_BLEND);
        renderer.set_draw_color_f32(Rgba::rgb(0.0, 0.0, 0.75));
        let _ = renderer.clear();

        let epoch = Instant::now();
        let res_ldr = ResourceLoader::new();
        let console = Console::new(&renderer, &ttf, epoch, res_ldr)?;

        Ok(Self {
            running: true,
            atlas: Atlas::new(),
            console,
            renderer,
            window,
            _ttf: TtfContext::new()?,
        })
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
        while self.running {
            // --- Processing ---
            let _ = self.renderer.clear();

            self.update_delta(delta.elapsed());
            delta = Instant::now();

            self.process_events();
            self.atlas.pack(&self.renderer);

            // --- Drawing ---
            self.console.draw(&self.renderer, &mut self.atlas);
            self.draw_atlas();

            let _ = self.renderer.present();
        }
    }

    fn process_events(&mut self) {
        for evt in EventIter::new() {
            match evt {
                Event::Quit => self.running = false,
                Event::KeyDown(k) => match k.key {
                    SDLK_F1 => self.console.switch(&mut self.atlas, &self.window),
                    SDLK_RETURN => self.process_command(),
                    other => self.console.process_key(other),
                },
                Event::TextInput(ti) => {
                    self.console.process_str(unsafe { c_ptr_to_str(ti.text) });
                }
                _ => (),
            }
        }
    }

    /// Command handling differs from the C++ version in that operations
    /// on the `Game` class need to be performed by itself, or we risk
    /// pissing off the borrow checker with multiple mutable borrows.
    fn process_command(&mut self) {
        if let ConsoleState::Enabled(ac) = &mut self.console.state {
            let mut args = ac.field.text.split(' ');
            if let Some(name) = args.next() {
                match name {
                    "exit" => {
                        self.running = false;
                    }
                    "test-args" => {
                        for (i, arg) in args.enumerate() {
                            println!("arg #{i} = \"{arg}\"");
                        }
                    }
                    "get-error" => println!("\"{}\"", { unsafe { halcyon::error::get_str() } }),
                    "set-error" => match args.next() {
                        Some(v) => halcyon::error::set(v),
                        None => println!("usage: set-error [value]"),
                    },
                    _ => println!("unknown command \"{name}\""),
                }

                ac.clear(&mut self.console.data);
            }
        }
    }

    fn update_delta(&mut self, elapsed: Duration) {
        self.console.update_delta(elapsed);
    }

    fn draw_atlas(&self) {
        if let Some(at) = self.atlas.texture.as_ref() {
            let sz = Rect::new(Point::new(300.0, 300.0), at.size());
            let _col = DrawColorGuard::new(&self.renderer, Rgba::BLACK);

            let _ = self.renderer.draw_rect(sz);
            let _ = self.renderer.draw(at, None, Some(&sz));
        }
    }
}
