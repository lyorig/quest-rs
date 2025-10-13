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
    prefix_id: AtlasId,
    line_id: AtlasId,
    should_repaint: bool,
}

impl ActiveConsole {
    pub fn new(prefix_id: AtlasId, line_id: AtlasId) -> Self {
        Self {
            field: Field::new(),
            prefix_id,
            line_id,
            should_repaint: true,
        }
    }

    fn set_cursor(&mut self, data: &mut CachedData) {
        data.outline.pos.x = data.tex_begin_crd + self.field.cursor as f32 * data.outline.size.x;
    }

    fn process_str(&mut self, data: &mut CachedData, input: &str) {
        self.should_repaint = self.field.process_str(input);

        self.field.trim_check();

        self.set_cursor(data);
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
                self.set_cursor(data)
            }
            FieldAction::CursorMoved => self.set_cursor(data),
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
            PointF32::new(data.tex_begin_crd, TEXT_OFFSET.y),
        );

        guard.set(Rgba::new(Rgb::WHITE, 0.5));
        let _ = rnd.fill_rect(data.outline);

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

    pub fn clear(&mut self, data: &mut CachedData) {
        self.field.clear();
        self.set_cursor(data);

        self.should_repaint = true;
    }
}

pub enum ConsoleState {
    Disabled,
    Enabled(ActiveConsole),
}

/// Not all data needs to be recreated for the
pub struct CachedData {
    placeholder_index: u8,
    font: Font,
    tex_begin_crd: f32,
    outline: RectF32,
}

/// Holds data required for Quest's console.
/// Also caches certain things for the activated variant.
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
        let outline = RectF32::new(
            Point::new(tex_begin_crd, TEXT_OFFSET.y),
            Text::new(&font, " ")?.size().into(),
        );

        Ok(Self {
            data: CachedData {
                placeholder_index: 0,
                font,
                tex_begin_crd,
                outline,
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

                self.state = ConsoleState::Enabled(ActiveConsole::new(prefix_id, line_id));
                self.data.outline.pos = Point::new(self.data.tex_begin_crd, TEXT_OFFSET.y);
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
    pub fn try_process_key(&mut self, k: SDL_Keycode) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.process_key(&mut self.data, k);
        }
    }

    pub fn try_process_str(&mut self, text: &str) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.process_str(&mut self.data, text);
        }
    }

    pub fn try_draw(&mut self, rnd: impl Into<RendererRef>, atlas: &mut Atlas) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.draw(&mut self.data, atlas, rnd);
        }
    }
}
