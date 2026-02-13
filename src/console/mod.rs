pub mod active;
mod cache;
pub mod command;
mod field;
pub mod state;
mod writer;

use std::time::{Duration, Instant};

use halcyon::{
    defs::SdlResult,
    rect::PointF32,
    renderer::RendererRef,
    resource_loader::ResourceLoader,
    ttf::{Text, TtfContext},
};

use sdl3_sys::keycode::*;

use crate::{
    console::{
        active::{ActiveConsole, TEXT_OFFSET},
        cache::CachedData,
        state::ConsoleState,
    },
    dprint,
    font::FontId,
    game::GameData,
    util::find_sized_font,
};

const PREFIX_TEXT: &str = "raine1@Arctic~ %";

pub struct Console {
    pub data: CachedData,
    pub state: ConsoleState,
}

impl Console {
    pub fn new<'a>(
        ttf: &'a TtfContext,
        rnd: RendererRef,
        epoch: Instant,
        base: ResourceLoader,
    ) -> SdlResult<Console> {
        let rs: PointF32 = rnd.output_size().into();

        let font = unsafe {
            find_sized_font(
                ttf,
                &base.resolve("../../bin/assets/UbuntuMono.ttf"),
                rs.y * 0.045,
            )
        }?;

        if !font.is_mono() {
            dprint!(
                epoch,
                "[INFO] <Console> Font \"{}\", isn't fixed-width",
                font.family()
            );
        }

        let padding_crd = rs.x * 0.015;
        let tex_begin_crd =
            TEXT_OFFSET.x + Text::new(&font, PREFIX_TEXT)?.size().x as f32 + padding_crd;

        Ok(Console {
            data: CachedData {
                placeholder_index: 0,
                input_x_origin: tex_begin_crd,
                history: Vec::new(),
            },
            state: ConsoleState::Disabled,
        })
    }

    pub fn switch(&mut self, data: &mut GameData) {
        let wnd = *data.window;

        match &self.state {
            ConsoleState::Disabled => {
                let _ = halcyon::keyboard::text_input_start(wnd);

                let np = self.data.next_placeholder();
                data.font_alloc(FontId::UBUNTU_MONO, np);
                data.font_alloc(FontId::UBUNTU_MONO, PREFIX_TEXT);

                self.state = ConsoleState::Enabled(ActiveConsole::new(&mut self.data));
            }

            ConsoleState::Enabled(ac) => {
                let _ = halcyon::keyboard::text_input_stop(wnd);

                data.font_free(FontId::UBUNTU_MONO, PREFIX_TEXT);
                data.font_free(
                    FontId::UBUNTU_MONO,
                    if ac.field.text.is_empty() {
                        self.data.current_placeholder()
                    } else {
                        &ac.field.text
                    },
                );

                self.state = ConsoleState::Disabled;
            }
        }
    }

    /// If the console is active, calls `ActiveConsole::process_key()`.
    /// Otherwise, does nothing.
    pub fn process_key(&mut self, data: &GameData, k: SDL_Keycode) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.process_key(&mut self.data, data, k);
        }
    }

    /// If the console is active, calls `ActiveConsole::process_str()`.
    /// Otherwise, does nothing.
    pub fn process_str(&mut self, data: &GameData, text: &str) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.process_str(&mut self.data, data, text);
        }
    }

    /// If the console is active, calls `ActiveConsole::draw()`.
    /// Otherwise, does nothing.
    pub fn draw(&mut self, data: &GameData) {
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
