use std::time::Instant;

use halcyon::{
    Result,
    color::Rgba,
    event::{Event, EventIter},
    gpu::Device,
    pixels::BlendMode,
    properties::Properties,
    rect::Point,
    renderer::Renderer,
    resource::Resource,
    traits::BlendModeable,
    ttf,
    util::c_ptr_to_str,
    window::Window,
};

use sdl3_sys::keycode::*;

use crate::{
    atlas::Atlas,
    chk,
    console::Console,
    font::store::FontStore,
    game::resources::Resources,
    ui::{Layer, ResizeInfo},
    util::{resource_loader::ResourceLoader, scheduler::Scheduler},
};

pub mod resources;

pub struct Game<'t> {
    pub data: Resources<'t>,
    sched: Scheduler<Resources<'t>>,
    console: Console,
}

impl Game<'_> {
    pub fn new<'t>(ttf: &'t ttf::Context) -> Result<Game<'t>> {
        let props = Properties::global()?;

        let wnd = Window::builder(props)
            .title(c"HalodaQuest")
            .size(Point::new(1280, 720))
            .position(Point::new(Window::POS_CENTERED, Window::POS_CENTERED))
            .resizable(true)
            .build_cleanup()?;

        let device = Device::builder(props)
            .debug_mode(false)
            .prefer_low_power(true)
            .shaders_metallib(true)
            .shaders_dxil(true)
            .build_cleanup()?;

        let rnd = Renderer::new_gpu(device.as_ref(), wnd.as_ref())?;

        // `SDL_CreateGPURenderer` defaults to uncapped presentation,
        // so the previous VSync behavior is restored manually.
        if !rnd.set_vsync(1) {
            return Err(halcyon::error::Error::current());
        }

        rnd.set_blend_mode(BlendMode::Blend);

        let res_ldr = ResourceLoader::from_pref()?;
        let mut atlas = Atlas::new();
        let store = FontStore::new(ttf, res_ldr, &mut atlas)?;
        let data = Resources::new(atlas, rnd, wnd, device, store);
        let console = Console::new(&data);
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
            self.data.renderer.set_draw_color_f32(Rgba::BLACK);

            chk!(self.data.renderer.clear());

            // --- Processing ---
            if !self.process_events() {
                break;
            }

            // --- Updating ---
            let now = Instant::now();
            let dt = now.duration_since(self.data.now);
            self.data.now = now;

            self.console.update_delta(dt);

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
                    SDLK_RETURN => self.process_command(),
                    other => self.console.process_key(&mut self.data, other),
                },
                Event::MouseWheelMotion(m) => {
                    if let Some(ref mut ac) = self.console.state {
                        ac.process_mouse(&mut self.data, &mut self.console.data, m);
                    }
                }
                Event::TextInput(ti) => {
                    let text = unsafe { c_ptr_to_str(ti.text) };
                    self.console.process_str(&mut self.data, text);
                }
                Event::WindowResized(r) => {
                    let ri = ResizeInfo::new(r);
                    self.console.resize(&ri, &mut self.data);
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

    pub fn quit() {
        chk!(Event::Quit.push());
    }
}
