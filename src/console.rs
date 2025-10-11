use std::time::{Duration, Instant};

use halcyon::{
    color::Rgba,
    defs::SdlResult,
    event::Event,
    guard::DrawColorGuard,
    rect::{Point, PointF32, RectF32},
    renderer::RendererRef,
    resource_loader::ResourceLoader,
    surface::Surface,
    ttf::{Font, FontRef, Text, TtfContext},
    util::c_ptr_to_str,
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

const MAX_CHARS: usize = 128;
const PREFIX_TEXT: &str = "raine1@Arctic~ %";
const CURSOR_BLINK_TIME: Duration = Duration::from_millis(500);
const TEXT_OFFSET: PointF32 = Point::new(10., 10.);

fn make_placeholder(font: impl Into<FontRef>, placeholder_index: u8) -> Surface {
    let font: FontRef = font.into();
    font.render_text_blended(
        PLACEHOLDERS[placeholder_index as usize],
        Rgba::rgb(0x80, 0x80, 0x80),
    )
    .unwrap()
}

pub struct ActiveConsole {
    field: Field,
    prefix_id: AtlasId,
    line_id: AtlasId,
    cursor_time: Duration,
    should_repaint: bool,
    is_cursor_visible: bool,
}

impl ActiveConsole {
    pub fn new(prefix_id: AtlasId, line_id: AtlasId) -> Self {
        Self {
            field: Field::new(),
            prefix_id,
            line_id,
            cursor_time: Duration::ZERO,
            should_repaint: true,
            is_cursor_visible: true,
        }
    }

    fn calculate_cursor(pos: usize, tbc: f32, mut cursor: RectF32) -> RectF32 {
        cursor.pos.x = tbc + (pos % MAX_CHARS) as f32 * cursor.size.x;
        cursor.pos.y = TEXT_OFFSET.y + cursor.size.y * (pos / MAX_CHARS) as f32;

        cursor
    }

    fn current_cursor(&self, tbc: f32, cursor: RectF32) -> RectF32 {
        Self::calculate_cursor(self.field.cursor, tbc, cursor)
    }

    pub fn set_cursor(&mut self, tbc: f32, outline: &mut RectF32) {
        *outline = Self::calculate_cursor(self.field.cursor, tbc, *outline);

        self.cursor_time = CURSOR_BLINK_TIME / 2;
        self.is_cursor_visible = true;
    }

    pub fn process_str(&mut self, input: &str, tbc: f32, outline: &mut RectF32) {
        self.should_repaint = self.field.process_str(input);

        self.set_cursor(tbc, outline);

        if self.field.text.len() > MAX_CHARS {
            self.field.trim(MAX_CHARS);
        }
    }

    /// Hand over a pressed key to this console's `Field`, it'll decide what to do next.
    /// Returns the rect that corresponds to the new outline.
    pub fn process_key(&mut self, tbc: f32, k: SDL_Keycode, cursor: RectF32) -> RectF32 {
        let op = self.field.process_key(k);

        if self.field.text.len() > MAX_CHARS {
            self.field.trim(MAX_CHARS);
        }

        // TODO: Convert to a "nicer" (not visually!) block with fallthroughs.
        match op {
            FieldAction::TextAdded | FieldAction::TextRemoved => {
                self.should_repaint = true;
                self.current_cursor(tbc, cursor)
            }
            FieldAction::CursorMoved => self.current_cursor(tbc, cursor),
            FieldAction::Noop => cursor,
        }
    }

    pub fn process_events(&mut self, events: &[Event], tex_begin_crd: f32, outline: &mut RectF32) {
        for evt in events {
            match evt {
                Event::TextInput(e) => {
                    let foo = unsafe { c_ptr_to_str(e.text) };
                    self.process_str(foo, tex_begin_crd, outline);
                }

                _ => (),
            }
        }
    }

    pub fn draw(
        &mut self,
        tbc: f32,
        m_outline: RectF32,
        font: impl Into<FontRef>,
        placeholder: u8,
        atlas: &mut Atlas,
        rnd: impl Into<RendererRef>,
    ) {
        let rnd: RendererRef = rnd.into();

        let guard = DrawColorGuard::new(rnd, Rgba::rgba(0., 0., 0., 0.5));
        let _ = rnd.fill_target();

        atlas.draw(rnd, self.prefix_id, TEXT_OFFSET);
        atlas.draw(rnd, self.line_id, PointF32::new(tbc, TEXT_OFFSET.y));

        if self.is_cursor_visible {
            guard.set(Rgba::rgba(1., 1., 1., 0.5));
            let _ = rnd.fill_rect(m_outline);
        }

        if self.should_repaint {
            self.should_repaint = false;
            atlas.replace(self.line_id, rnd, self.make_line(font, placeholder));
        }
    }

    pub fn make_line(&self, font: impl Into<FontRef>, placeholder: u8) -> Surface {
        if self.field.text.is_empty() {
            make_placeholder(font, placeholder)
        } else {
            let font: FontRef = font.into();
            font.render_text_blended(&self.field.text, Rgba::rgb(255, 255, 255))
                .unwrap()
        }
    }
}

pub enum ConsoleState {
    Disabled,
    Enabled(ActiveConsole),
}

/// Holds data required for Quest's console.
/// Also caches certain things for the activated variant.
pub struct Console {
    placeholder_index: u8,

    font: Font,

    padding_crd: f32,
    pub tex_begin_crd: f32,

    wrap_len: f32,
    pub outline: RectF32,

    line_chars: u8,
    pub state: ConsoleState,

    should_repaint: bool,
    is_cursor_visible: bool,
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
        let wrap_len = rs.x - tex_begin_crd - padding_crd;
        let outline = RectF32::new(
            Point::new(tex_begin_crd, TEXT_OFFSET.y),
            Text::new(&font, " ")?.size().into(),
        );

        Ok(Self {
            placeholder_index: 0,

            font,

            padding_crd,
            tex_begin_crd,

            wrap_len,
            outline,

            line_chars: (wrap_len / outline.size.x) as _,
            state: ConsoleState::Disabled,

            should_repaint: false,
            is_cursor_visible: false,
        })
    }

    pub fn switch(&mut self, atlas: &mut Atlas, wnd: impl Into<WindowRef>) {
        match &self.state {
            ConsoleState::Disabled => {
                let _ = halcyon::keyboard::text_input_start(wnd);
                self.is_cursor_visible = true;

                let prefix_id = atlas.push(
                    self.font
                        .render_text_blended(PREFIX_TEXT, Rgba::rgb(0, 255, 0))
                        .unwrap(),
                );

                let line_id = atlas.push(self.make_placeholder());

                self.state = ConsoleState::Enabled(ActiveConsole::new(prefix_id, line_id));
                self.outline.pos = Point::new(self.tex_begin_crd, TEXT_OFFSET.y);
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
            self.outline = ac.process_key(self.tex_begin_crd, k, self.outline);
        }
    }

    pub fn try_draw(&mut self, rnd: impl Into<RendererRef>, atlas: &mut Atlas) {
        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.draw(
                self.tex_begin_crd,
                self.outline,
                &self.font,
                self.placeholder_index,
                atlas,
                rnd,
            );
        }
    }

    pub fn process_events(
        &mut self,
        events: &[Event],
        atlas: &mut Atlas,
        wnd: impl Into<WindowRef> + Copy,
    ) {
        // If multiple F1 presses somehow occur in a single frame,
        // this ensures the console is only switched once.
        let mut should_switch = false;

        for evt in events {
            match evt {
                Event::KeyDown(k) => match k.key {
                    SDLK_F1 => should_switch = !should_switch,
                    k => self.try_process_key(k),
                },
                _ => (),
            }
        }

        if let ConsoleState::Enabled(ac) = &mut self.state {
            ac.process_events(events, self.tex_begin_crd, &mut self.outline);
        }

        if should_switch {
            self.switch(atlas, wnd);
        }
    }

    fn make_placeholder(&mut self) -> Surface {
        self.placeholder_index += 1;
        if self.placeholder_index as usize == PLACEHOLDERS.len() {
            self.placeholder_index = 0;
        }

        make_placeholder(&self.font, self.placeholder_index)
    }
}
