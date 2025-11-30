pub mod active;
pub mod cache;
pub mod state;
pub mod writer;

use std::time::{Duration, Instant};

use halcyon::{
    color::Rgba,
    defs::SdlResult,
    rect::PointF32,
    renderer::RendererRef,
    resource_loader::ResourceLoader,
    ttf::{Text, TtfContext},
    window::WindowRef,
};

use sdl3_sys::keycode::*;

use crate::{
    atlas::Atlas,
    console::{
        active::{ActiveConsole, TEXT_OFFSET, make_placeholder},
        cache::CachedData,
        state::ConsoleState,
    },
    dprint,
    glyph_map::GlyphMap,
    util::find_sized_font,
};

const PREFIX_TEXT: &str = "raine1@Arctic~ %";

pub struct Console<'a> {
    pub data: CachedData<'a>,
    pub state: ConsoleState,
}

impl Console<'_> {
    pub fn new<'a>(
        ttf: &'a TtfContext,
        rnd: impl Into<RendererRef>,
        epoch: Instant,
        base: ResourceLoader,
    ) -> SdlResult<Console<'a>> {
        let rnd: RendererRef = rnd.into();
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
        let glyph_size = Text::new(&font, " ")?.size().into();

        Ok(Console::<'a> {
            data: CachedData::<'a> {
                placeholder_index: 0,
                font,
                input_x_origin: tex_begin_crd,
                glyph_size,
                glyph_map: GlyphMap::new(),
                history: Vec::new(),
            },
            state: ConsoleState::Disabled,
        })
    }

    pub fn switch(&mut self, atlas: &mut Atlas, wnd: impl Into<WindowRef>) {
        match &self.state {
            ConsoleState::Disabled => {
                let _ = halcyon::keyboard::text_input_start(wnd);

                let prefix_id = atlas.push(
                    self.data
                        .font
                        .render_text_blended(PREFIX_TEXT, Rgba::GREEN)
                        .unwrap(),
                );

                let line_id = atlas.push(make_placeholder(&mut self.data));

                self.state =
                    ConsoleState::Enabled(ActiveConsole::new(&self.data, prefix_id, line_id));
            }

            ConsoleState::Enabled(ac) => {
                let _ = halcyon::keyboard::text_input_stop(wnd);

                atlas.remove(ac.prefix_id);
                atlas.remove(ac.line_id);

                self.state = ConsoleState::Disabled;
            }
        }
    }

    /// If the console is active, calls `ActiveConsole::process_key()`.
    /// Otherwise, does nothing.
    pub fn process_key(&mut self, k: SDL_Keycode) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.process_key(&mut self.data, k);
        }
    }

    /// If the console is active, calls `ActiveConsole::process_str()`.
    /// Otherwise, does nothing.
    pub fn process_str(&mut self, text: &str) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.process_str(&mut self.data, text);
        }
    }

    /// If the console is active, calls `ActiveConsole::draw()`.
    /// Otherwise, does nothing.
    pub fn draw(&mut self, rnd: impl Into<RendererRef>, atlas: &mut Atlas) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.draw(&mut self.data, atlas, rnd);
        }
    }

    pub fn update_delta(&mut self, elapsed: Duration) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.update_delta(elapsed);
        }
    }
}
