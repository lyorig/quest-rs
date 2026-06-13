pub mod active;
mod cache;
pub mod command;
mod field;
pub mod state;
mod writer;

use std::time::Duration;

use halcyon::{
    defs::SdlResult,
    renderer::Renderer,
    traits::{Ref, Resource},
};

use sdl3_sys::keycode::*;

use crate::{
    chk,
    console::{active::ActiveConsole, cache::CachedData, state::ConsoleState},
    font::store::FontId,
    game::resources::GameResources,
};

const PREFIX_TEXT: &str = "raine1@Arctic~ %";
const CONSOLE_FONT: FontId = FontId::UBUNTU_MONO;

pub struct Console {
    pub data: CachedData,
    pub state: ConsoleState,
}

impl Console {
    pub fn new(rnd: Ref<Renderer>) -> SdlResult<Console> {
        Ok(Console {
            data: CachedData::new(rnd),
            state: ConsoleState::Disabled,
        })
    }

    pub fn switch(&mut self, data: &mut GameResources) {
        let wnd = data.window.as_ref();

        match &self.state {
            ConsoleState::Disabled => {
                chk!(halcyon::keyboard::text_input_start(wnd));

                let np = self.data.next_placeholder();
                data.font_alloc(CONSOLE_FONT, np);
                data.font_alloc(CONSOLE_FONT, PREFIX_TEXT);
                data.font_alloc(CONSOLE_FONT, self.data.writer.data());

                self.state = ConsoleState::Enabled(ActiveConsole::new(&mut self.data));
            }

            ConsoleState::Enabled(ac) => {
                chk!(halcyon::keyboard::text_input_stop(wnd));

                data.font_free(CONSOLE_FONT, self.data.writer.data());
                data.font_free(CONSOLE_FONT, PREFIX_TEXT);
                data.font_free(CONSOLE_FONT, self.data.current_placeholder());
                data.font_free(CONSOLE_FONT, &ac.field.text);

                // NOTE: Only for debug purposes.
                data.font_gc(CONSOLE_FONT);

                self.state = ConsoleState::Disabled;
            }
        }
    }

    /// If the console is active, calls `ActiveConsole::process_key()`.
    /// Otherwise, does nothing.
    pub fn process_key(&mut self, data: &mut GameResources, k: SDL_Keycode) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.process_key(&mut self.data, data, k);
        }
    }

    /// If the console is active, calls `ActiveConsole::process_str()`.
    /// Otherwise, does nothing.
    pub fn process_str(&mut self, data: &mut GameResources, text: &str) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.process_str(&mut self.data, data, text);
        }
    }

    /// If the console is active, calls `ActiveConsole::draw()`.
    /// Otherwise, does nothing.
    pub fn draw(&mut self, data: &GameResources) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.draw(&mut self.data, data);
        }
    }

    pub fn update_delta(&mut self, elapsed: Duration) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.update_delta(elapsed);
        }
    }
}
