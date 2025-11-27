use std::time::{Duration, Instant};

use halcyon::{
    color::Rgba,
    defs::SdlResult,
    event::{Event, EventIter},
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
    command::{self},
    console::{Console, ConsoleState},
};

pub struct GameData {
    pub running: bool,

    pub atlas: Atlas,

    pub renderer: Renderer,
    pub window: Window,

    pub _ttf: TtfContext,
}

impl GameData {
    pub fn draw_atlas(&self) {
        if let Some(at) = self.atlas.texture.as_ref() {
            let sz = Rect::new(Point::new(300.0, 300.0), at.size());

            self.renderer.set_draw_color_f32(Rgba::BLACK);
            let _ = self.renderer.draw_rect(sz);
            let _ = self.renderer.draw(at, None, Some(&sz));
        }
    }
}

pub struct Game {
    pub data: GameData,
    console: Console,
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

        let epoch = Instant::now();
        let res_ldr = ResourceLoader::new();
        let console = Console::new(&renderer, &ttf, epoch, res_ldr)?;

        Ok(Self {
            data: GameData {
                running: true,
                atlas: Atlas::new(),
                renderer,
                window,
                _ttf: TtfContext::new()?,
            },
            console,
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
        while self.data.running {
            self.data
                .renderer
                .set_draw_color_f32(Rgba::rgb(0.0, 0.0, 0.75));
            let _ = self.data.renderer.clear();

            // --- Processing ---
            self.process_events();

            // --- Updating ---
            self.update_delta(delta.elapsed());
            delta = Instant::now();

            // --- Drawing ---
            self.console.draw(&self.data.renderer, &mut self.data.atlas);
            self.data.draw_atlas();

            self.data.atlas.pack(&self.data.renderer);
            let _ = self.data.renderer.present();
        }
    }

    fn process_events(&mut self) {
        for evt in EventIter::new() {
            match evt {
                Event::Quit => self.data.running = false,
                Event::KeyDown(k) => match k.key {
                    SDLK_F1 => self.console.switch(&mut self.data.atlas, &self.data.window),
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

    fn process_command(&mut self) {
        if let ConsoleState::Enabled(ac) = &mut self.console.state {
            let mut args = ac.field.text.split(' ');
            if let Some(name) = args.next() {
                match name {
                    c => match command::find(c) {
                        Some(c) => c.execute(&mut self.data, args),
                        None => println!("unknown command \"{name}\""),
                    },
                }

                ac.clear(&mut self.console.data);
            }
        }
    }

    fn update_delta(&mut self, elapsed: Duration) {
        self.console.update_delta(elapsed);
    }
}
