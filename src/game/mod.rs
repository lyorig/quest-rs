use std::{
    path::Path,
    time::{Duration, Instant},
};

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
    keycode::{SDLK_F1, SDLK_P, SDLK_RETURN},
};

use crate::{
    atlas::Atlas, chk, console::Console, font::store::FontStore, game::resources::Resources,
    util::scheduler::Scheduler,
};

pub mod resources;

pub struct Game<'t> {
    pub data: Resources<'t>,
    sched: Scheduler<Resources<'t>>,
    console: Console,
}

impl Game<'_> {
    pub fn new<'t>(ttf: &'t ttf::Context) -> SdlResult<Game<'t>> {
        let window = WindowBuilder::new()
            .size(Point::new(1280, 720))
            .position(Point::new(Window::POS_CENTERED, Window::POS_CENTERED))
            .title(c"HalodaQuest")
            .build()?;

        let renderer = RendererBuilder::new(window.as_ref()).vsync(1).build()?;
        renderer.set_blend_mode(SDL_BLENDMODE_BLEND);

        let res_ldr = ResourceLoader::from_path(Path::new("/usr/local/share/quest"));
        let data = Resources::new(Atlas::new(), renderer, window, FontStore::new(ttf, res_ldr));
        let console = Console::new();
        let sched = Scheduler::new();

        Ok(Game {
            data,
            sched,
            console,
        })
    }

    pub fn main_loop(&mut self) {
        loop {
            let old = self.data.renderer.draw_color_f32();
            self.data
                .renderer
                .set_draw_color_f32(Rgba::rgb(0.0, 0.0, 0.75));

            chk!(self.data.renderer.clear());

            // --- Processing ---
            if !self.process_events() {
                break;
            }

            // --- Updating ---
            self.update_delta();
            self.data.now = Instant::now();

            self.sched.update(self.data.now, &mut self.data);

            self.data.atlas.pack(self.data.renderer.as_ref());

            // --- Drawing ---
            self.console.draw(&self.data);

            chk!(self.data.renderer.present());

            self.data.renderer.set_draw_color_f32(old);
        }
    }

    fn process_events(&mut self) -> bool {
        for evt in EventIter::new() {
            match evt {
                Event::Quit => return false,
                Event::KeyDown(k) => match k.key {
                    SDLK_F1 => self.console.switch(&mut self.data),
                    SDLK_P => self
                        .sched
                        .schedule(self.data.now + Duration::from_secs(1), |_| println!("Blah")),
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

        true
    }

    fn process_command(&mut self) {
        if let Some(ref mut ac) = self.console.state {
            ac.process_command(&mut self.console.data, &mut self.data);
        }
    }

    fn update_delta(&mut self) {
        self.console.update_delta(self.data.now.elapsed());
    }

    pub fn quit() {
        chk!(Event::Quit.push());
    }
}
