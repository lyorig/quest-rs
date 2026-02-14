pub mod active;
mod cache;
pub mod command;
mod field;
pub mod state;
mod writer;

use std::time::Duration;

use halcyon::{defs::SdlResult, rect::PointF32, renderer::RendererRef};

use sdl3_sys::keycode::*;

use crate::{
    console::{active::ActiveConsole, cache::CachedData, state::ConsoleState},
    font::store::FontId,
    game::resources::GameResources,
};

const PREFIX_TEXT: &str = "raine1@Arctic~ %";

pub struct Console {
    pub data: CachedData,
    pub state: ConsoleState,
}

impl Console {
    pub fn new<'a>(_rnd: RendererRef) -> SdlResult<Console> {
        // TODO: Use the renderer.
        let glyph_size = PointF32::new(16.0, 32.0);

        Ok(Console {
            data: CachedData::new(glyph_size),
            state: ConsoleState::Disabled,
        })
    }

    pub fn switch(&mut self, data: &mut GameResources) {
        let wnd = *data.window;

        match &self.state {
            ConsoleState::Disabled => {
                let _ = halcyon::keyboard::text_input_start(wnd);

                let np = self.data.next_placeholder();
                data.font_alloc(FontId::UBUNTU_MONO, np);
                data.font_alloc(FontId::UBUNTU_MONO, PREFIX_TEXT);
                data.font_alloc(FontId::UBUNTU_MONO, self.data.writer.data());

                self.state = ConsoleState::Enabled(ActiveConsole::new(&mut self.data));
            }

            ConsoleState::Enabled(ac) => {
                let _ = halcyon::keyboard::text_input_stop(wnd);

                data.font_free(FontId::UBUNTU_MONO, self.data.writer.data());
                data.font_free(FontId::UBUNTU_MONO, PREFIX_TEXT);
                data.font_free(
                    FontId::UBUNTU_MONO,
                    if ac.field.text.is_empty() {
                        self.data.current_placeholder()
                    } else {
                        &ac.field.text
                    },
                );

                // NOTE: Only for debug purposes.
                data.font_gc(FontId::UBUNTU_MONO);

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
