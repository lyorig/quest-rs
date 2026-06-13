use std::time::Instant;

use halcyon::{
    color::Rgba,
    defs::SdlResult,
    event::Event,
    rect::Point,
    renderer::RendererBuilder,
    resource_loader::ResourceLoader,
    traits::{BlendMode, Resource},
    util::c_ptr_to_str,
    window::{Window, WindowBuilder},
};

use sdl3_main::AppResult;
use sdl3_sys::{
    blendmode::SDL_BLENDMODE_BLEND,
    keycode::{SDLK_F1, SDLK_RETURN},
};

use crate::{
    atlas::Atlas,
    chk,
    console::{Console, state::ConsoleState},
    dprintln,
    font::store::FontStore,
    game::resources::GameResources,
};

pub mod resources;

pub struct Game {
    pub data: GameResources,
    console: Console,
    delta: Instant,
}

impl Game {
    pub fn new() -> SdlResult<Self> {
        let window = WindowBuilder::new()
            .size(Point::new(1280, 720))
            .position(Point::new(Window::POS_CENTERED, Window::POS_CENTERED))
            .title(c"HalodaQuest [Euclid]")
            .build()?;

        let mut renderer = RendererBuilder::new(window.as_ref());
        if !std::env::args().any(|x| x == "--no-vsync") {
            renderer.vsync(1);
        }

        let renderer = renderer.build()?;
        renderer.set_blend_mode(SDL_BLENDMODE_BLEND);

        let res_ldr = ResourceLoader::new();
        let data = GameResources::new(Atlas::new(), renderer, window, unsafe {
            FontStore::new(res_ldr)
        });

        let console = Console::new(data.renderer.as_ref())?;
        let delta = Instant::now();

        dprintln!("Game init complete");

        Ok(Game {
            data,
            console,
            delta,
        })
    }

    pub fn iterate(&mut self) -> AppResult {
        let old = self.data.renderer.draw_color_f32();
        self.data
            .renderer
            .set_draw_color_f32(Rgba::rgb(0.0, 0.0, 0.75));

        chk!(self.data.renderer.clear());

        // --- Updating ---
        self.update_delta();
        self.delta = Instant::now();

        self.data.atlas.pack(self.data.renderer.as_ref());

        // --- Drawing ---
        self.console.draw(&self.data);

        chk!(self.data.renderer.present());

        self.data.renderer.set_draw_color_f32(old);

        AppResult::Continue
    }

    pub fn process_event(&mut self, evt: Event) -> AppResult {
        match evt {
            Event::Quit => return AppResult::Success,
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

        AppResult::Continue
    }

    fn process_command(&mut self) {
        if let ConsoleState::Enabled(ac) = &mut self.console.state {
            ac.process_command(&mut self.console.data, &mut self.data);
        }
    }

    fn update_delta(&mut self) {
        let elapsed = self.delta.elapsed();
        self.console.update_delta(elapsed);
        self.delta = Instant::now();
    }
}
