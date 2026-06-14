use std::time::{Duration, Instant};

use halcyon::{
    color::Rgba,
    defs::SdlResult,
    event::{Event, EventIter},
    rect::Point,
    renderer::RendererBuilder,
    resource_loader::ResourceLoader,
    traits::{BlendMode, Resource},
    ttf,
    util::c_ptr_to_str,
    window::{Window, WindowBuilder},
};

use sdl3_sys::{
    blendmode::SDL_BLENDMODE_BLEND,
    keycode::{SDLK_F1, SDLK_RETURN},
};

use crate::{
    atlas::Atlas,
    chk,
    console::{Console, state::ConsoleState},
    font::store::FontStore,
    game::resources::GameResources,
};

pub mod resources;

pub struct Game<'t> {
    pub data: GameResources<'t>,
    console: Console,
}

impl Game<'_> {
    /// Create a new game.
    pub fn new<'t>(ttf: &'t ttf::Context) -> SdlResult<Game<'t>> {
        let window = WindowBuilder::new()
            .size(Point::new(1280, 720))
            .position(Point::new(Window::POS_CENTERED, Window::POS_CENTERED))
            .title(c"HalodaQuest [Euclid]")
            .build()?;

        let renderer = RendererBuilder::new(window.as_ref()).vsync(1).build()?;
        renderer.set_blend_mode(SDL_BLENDMODE_BLEND);

        let res_ldr = ResourceLoader::new();
        let data = GameResources::new(Atlas::new(), renderer, window, FontStore::new(ttf, res_ldr));
        let console = Console::new();

        Ok(Game { data, console })
    }

    /// Enter the main loop.
    pub fn main_loop(&mut self) {
        let mut delta = Instant::now();

        while self.data.running {
            let old = self.data.renderer.draw_color_f32();
            self.data
                .renderer
                .set_draw_color_f32(Rgba::rgb(0.0, 0.0, 0.75));

            chk!(self.data.renderer.clear());

            // --- Processing ---
            self.process_events();

            // --- Updating ---
            self.update_delta(delta.elapsed());
            delta = Instant::now();

            self.data.atlas.pack(self.data.renderer.as_ref());

            // --- Drawing ---
            self.console.draw(&self.data);

            chk!(self.data.renderer.present());

            self.data.renderer.set_draw_color_f32(old);
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
                    let text = unsafe { c_ptr_to_str(ti.text) };
                    self.console.process_str(&mut self.data, text);
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
