use std::time::Instant;

use halcyon::{
    color::{Rgb, Rgba},
    defs::SdlResult,
    guard::DrawColorGuard,
    rect::{Point, PointF32, RectF32},
    renderer::RendererRef,
    resource_loader::ResourceLoader,
    surface::Surface,
    ttf::{Font, Text, TtfContext},
    window::WindowRef,
};
use sdl3_sys::keycode::*;

use crate::{
    atlas::{Atlas, AtlasId},
    dprint,
    field::{Field, FieldAction},
    util::find_sized_font,
};

const PLACEHOLDERS: [&str; 40] = [
    "[meow]",
    "[redacted]",
    "[your turn]",
    "[womp womp]",
    "[one big CVE]",
    "[kevin's heart]",
    "[lods of emone]",
    "[be not afraid]",
    "[see you again]",
    "[forget me not]",
    "[all is pretty]",
    "[sudo deez nuts]",
    "[openest source]",
    "[at your service]",
    "[with eye serene]",
    "[is anyone there?]",
    "[food for thought]",
    "[made with Halcyon]",
    "[49.0481N, 17.4838E]",
    "[are you satisfied?]",
    "[enter command here]",
    "[running out of time]",
    "[not actually random]",
    "[watch?v=lo5cG0FhWro]",
    "[not POSIX compliant]",
    "[start typing, please]",
    "[commands not included]",
    "[segfaulting since 2021]",
    "[waiting for user input]",
    "[non-euclidean interface]",
    "[who needs documentation]",
    "[sudo pacman -S lyofetch]",
    "[no man page here, sorry]",
    "[ševalicious out tomorrow]",
    "[licensed under the WTFPL]",
    "[streets and sodium lights]",
    "[quoth the raven, nevermore]",
    "[docker? I barely know 'er!]",
    "[rm -rf / --no-preserve-root]",
    "[MSVC is the real final boss]",
];

pub const MAX_CHARS: usize = 32;
const PREFIX_TEXT: &str = "raine1@Arctic~ %";
const TEXT_OFFSET: PointF32 = Point::new(10.0, 10.0);

fn make_placeholder(data: &mut CachedData) -> Surface {
    let ret = data
        .font
        .render_text_blended(
            PLACEHOLDERS[data.placeholder_index as usize],
            Rgba::rgb(0x80, 0x80, 0x80),
        )
        .unwrap();

    data.placeholder_index = (data.placeholder_index + 1) % PLACEHOLDERS.len() as u8;

    ret
}

pub struct ActiveConsole {
    pub field: Field,

    /// Where the cursor is currently being drawn to.
    /// Only updated when `self.update_outline()` is called,
    /// which sets its location to correspond to the `Field` cursor.
    cursor_pos: PointF32,

    prefix_id: AtlasId,
    line_id: AtlasId,
    should_repaint: bool,
}

impl ActiveConsole {
    pub fn new(data: &CachedData, prefix_id: AtlasId, line_id: AtlasId) -> Self {
        Self {
            field: Field::new(),
            cursor_pos: PointF32::new(data.input_x_origin, TEXT_OFFSET.y),
            prefix_id,
            line_id,
            should_repaint: true,
        }
    }

    fn update_outline(&mut self, data: &mut CachedData) {
        self.cursor_pos.x = data.input_x_origin + self.field.cursor as f32 * data.glyph_size.x;
    }

    fn process_str(&mut self, data: &mut CachedData, input: &str) {
        self.should_repaint = self.field.process_str(input);
        self.field.trim_check();
        self.update_outline(data);
    }

    /// Hands over a pressed key to this console's `Field`, which decides what to do next.
    /// Returns the rect that corresponds to the new outline.
    fn process_key(&mut self, data: &mut CachedData, k: SDL_Keycode) {
        let op = self.field.process_key(k);

        self.field.trim_check();

        // TODO: Convert to a "nicer" (not visually!) block with fallthroughs.
        match op {
            FieldAction::TextAdded | FieldAction::TextRemoved => {
                self.should_repaint = true;
                self.update_outline(data)
            }
            FieldAction::CursorMoved => self.update_outline(data),
            FieldAction::Noop => (),
        }
    }

    fn draw(&mut self, data: &mut CachedData, atlas: &mut Atlas, rnd: impl Into<RendererRef>) {
        let rnd: RendererRef = rnd.into();

        let guard = DrawColorGuard::new(rnd, Rgba::new(Rgb::BLACK, 0.5));
        let _ = rnd.fill_target();

        atlas.draw(rnd, self.prefix_id, TEXT_OFFSET);
        atlas.draw(
            rnd,
            self.line_id,
            PointF32::new(data.input_x_origin, TEXT_OFFSET.y),
        );

        guard.set(Rgba::new(Rgb::WHITE, 0.5));
        let _ = rnd.fill_rect(RectF32::new(self.cursor_pos, data.glyph_size));

        if self.should_repaint {
            self.should_repaint = false;

            atlas.replace(self.line_id, rnd, self.make_line(data));
        }
    }

    fn make_line(&self, data: &mut CachedData) -> Surface {
        if self.field.text.is_empty() {
            make_placeholder(data)
        } else {
            data.font
                .render_text_blended(&self.field.text, Rgba::WHITE)
                .unwrap()
        }
    }

    /// Clear the `Field` and update the cursor,
    /// and signal for a repaint.
    pub fn clear(&mut self, data: &mut CachedData) {
        self.field.clear();
        self.update_outline(data);

        self.should_repaint = true;
    }
}

pub enum ConsoleState {
    Disabled,
    Enabled(ActiveConsole),
}

/// Not all data needs to be recreated every time the console is activated
/// (i.e. on every `ActiveConsole::new()`). This struct aims to achieve just
/// that, while also preventing double-mutable-borrow errors that would otherwise
/// occur if the calling `Console` passed itself as a parameter.
pub struct CachedData {
    placeholder_index: u8,

    font: Font,

    /// The X coordinate of the input itself, equal to (placeholder names):
    /// `left_prefix_padding + prefix_length + right_prefix_padding`
    input_x_origin: f32,

    /// This is cached because the size component is the size of
    /// one glyph, and since `Self::font` is cached as well, we don't
    /// need to re-calculate it on every console activation.
    glyph_size: PointF32,
}

pub struct Console {
    pub data: CachedData,
    pub state: ConsoleState,
}

impl Console {
    pub fn new(
        rnd: impl Into<RendererRef>,
        ttf: &TtfContext,
        epoch: Instant,
        base: ResourceLoader,
    ) -> SdlResult<Self> {
        let rnd: RendererRef = rnd.into();
        let rs: PointF32 = rnd.output_size().into();

        let font = find_sized_font(
            ttf,
            &base.resolve("../../bin/assets/UbuntuMono.ttf"),
            rs.y * 0.045,
        )?;

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

        Ok(Self {
            data: CachedData {
                placeholder_index: 0,
                font,
                input_x_origin: tex_begin_crd,
                glyph_size,
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
}
