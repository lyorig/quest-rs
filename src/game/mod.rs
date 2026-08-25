use std::time::Instant;

use halcyon::{
    Result,
    color::Rgba,
    event::{Event, EventIter},
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
    font::store::FontStore,
    game::resources::Resources,
    util::{resource_loader::ResourceLoader, scheduler::Scheduler},
};

pub mod resources;

pub struct Game<'t> {
    pub data: Resources<'t>,
    sched: Scheduler<Resources<'t>>,
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

        let rnd = Renderer::builder(props)
            .window(wnd.as_ref())
            .vsync(1)
            .build_cleanup()?;

        rnd.set_blend_mode(BlendMode::Blend);

        let res_ldr = ResourceLoader::from_pref()?;
        let mut atlas = Atlas::new();
        let store = FontStore::new(ttf, res_ldr, &mut atlas)?;
        let data = Resources::new(atlas, rnd, wnd, store);
        let sched = Scheduler::new();

        Ok(Game { data, sched })
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

            if let Some(ref mut c) = self.data.console {
                c.update_delta(dt);
            }

            self.sched.update(self.data.now, &mut self.data);

            self.data.atlas.pack(self.data.renderer.as_ref());

            if let Some(ref mut c) = self.data.console {
                c.draw(
                    &mut self.data.console_cache,
                    self.data.renderer.as_ref(),
                    &mut self.data.fonts,
                    &mut self.data.atlas,
                );
            }

            chk!(self.data.renderer.present());

            self.data.renderer.set_draw_color_f32(old);
        }
    }

    fn process_events(&mut self) -> bool {
        for evt in EventIter::new() {
            match evt {
                Event::Quit => return false,
                Event::KeyDown(k) => match k.key {
                    SDLK_F1 => self.data.toggle_console(),
                    SDLK_RETURN => self.process_command(),
                    other => {
                        if let Some(ref mut c) = self.data.console {
                            c.process_key(
                                &mut self.data.console_cache,
                                &mut self.data.fonts,
                                &mut self.data.atlas,
                                other,
                            );
                        }
                    }
                },
                Event::MouseWheelMotion(m) => {
                    if let Some(ref mut c) = self.data.console {
                        c.process_mouse(
                            self.data.renderer.as_ref(),
                            &mut self.data.console_cache,
                            m,
                        );
                    }
                }
                Event::TextInput(ti) => {
                    let text = unsafe { c_ptr_to_str(ti.text) };
                    if let Some(ref mut c) = self.data.console {
                        c.process_str(
                            &mut self.data.console_cache,
                            &mut self.data.fonts,
                            &mut self.data.atlas,
                            text,
                        );
                    }
                }
                _ => (),
            }
        }

        true
    }

    fn process_command(&mut self) {
        self.data.console = self.data.console.take().map(|mut c| {
            c.process_command(&mut self.data);
            c
        });
    }

    pub fn quit() {
        chk!(Event::Quit.push());
    }
}
