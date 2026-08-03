mod cache;
mod command;
mod field;
mod inner;
mod writer;

use std::time::Duration;

use halcyon::resource::Resource;

use sdl3_sys::keycode::*;

use crate::{
    chk,
    console::{cache::CachedData, inner::Inner},
    font::store::FontId,
    game::resources::Resources,
    ui,
};

const PREFIX_TEXT: &str = "raine1@Arctic~ %";
const CONSOLE_FONT: FontId = FontId::UBUNTU_MONO;

pub struct Console {
    pub data: CachedData,
    pub state: Option<Inner>,
}

impl Console {
    pub fn new(res: &Resources) -> Console {
        Console {
            data: CachedData::new(res),
            state: None,
        }
    }

    pub fn switch(&mut self, data: &mut Resources) {
        let wnd = data.window.as_ref();

        match &self.state {
            None => {
                chk!(halcyon::keyboard::text_input_start(wnd));

                let np = self.data.next_placeholder();
                data.font_alloc(CONSOLE_FONT, np);
                data.font_alloc(CONSOLE_FONT, PREFIX_TEXT);
                data.font_alloc(CONSOLE_FONT, self.data.writer.data());
                command::help_iter().for_each(|s| data.font_alloc(CONSOLE_FONT, s));

                self.state = Some(Inner::new(&mut self.data));
            }

            Some(ac) => {
                chk!(halcyon::keyboard::text_input_stop(wnd));

                data.font_free(CONSOLE_FONT, self.data.writer.data());
                data.font_free(CONSOLE_FONT, PREFIX_TEXT);
                data.font_free(CONSOLE_FONT, self.data.current_placeholder());
                data.font_free(CONSOLE_FONT, &ac.field.text);
                command::help_iter().for_each(|s| data.font_free(CONSOLE_FONT, s));

                self.state = None;
            }
        }
    }

    /// If the console is active, calls [`Inner::process_key`].
    /// Otherwise, does nothing.
    pub fn process_key(&mut self, data: &mut Resources, k: SDL_Keycode) {
        if let Some(ac) = &mut self.state {
            ac.process_key(&mut self.data, data, k);
        }
    }

    /// If the console is active, calls [`Inner::process_str`].
    /// Otherwise, does nothing.
    pub fn process_str(&mut self, data: &mut Resources, text: &str) {
        if let Some(ac) = &mut self.state {
            ac.process_str(&mut self.data, data, text);
        }
    }

    /// If the console is active, calls [`Inner::draw`].
    /// Otherwise, does nothing.
    pub fn draw(&mut self, data: &Resources) {
        if let Some(ac) = &mut self.state {
            ac.draw(&mut self.data, data);
        }
    }

    pub fn update_delta(&mut self, elapsed: Duration) {
        if let Some(ac) = &mut self.state {
            ac.update_delta(elapsed);
        }
    }
}

impl ui::Layer for Console {
    fn resize(&mut self, layout: &ui::ResizeInfo, res: &mut Resources) {
        self.data.resize(layout.new_size, res);
    }
}
