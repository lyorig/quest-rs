use std::time::{Duration, Instant};

use halcyon::{
    color::Rgba,
    defs::SdlResult,
    event::Event,
    rect::{Point, PointF32, RectF32},
    renderer::RendererRef,
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
    resource_loader::ResourceLoader,
    util::find_sized_font,
};

const PLACEHOLDERS: [&str; 39] = [
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

fn make_placeholder(font: impl Into<FontRef>, placeholder_index: usize) -> Surface {
    let font: FontRef = font.into();
    font.render_text_blended(PLACEHOLDERS[placeholder_index], Rgba::rgb(0x80, 0x80, 0x80))
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

    pub fn set_cursor(&mut self, cons: &mut Console) {
        cons.outline = Self::calculate_cursor(self.field.cursor, cons.tex_begin_crd, cons.outline);

        self.cursor_time = CURSOR_BLINK_TIME / 2;
        self.is_cursor_visible = true;
    }

    pub fn process_str(&mut self, input: &str) {
        self.should_repaint = self.field.process_str(input);

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
            }
            _ => return cursor,
        }

        self.current_cursor(tbc, cursor)
    }

    pub fn draw(
        &mut self,
        tbc: f32,
        m_wrap: f32,
        m_outline: RectF32,
        font: impl Into<FontRef>,
        placeholder: usize,
        atlas: &mut Atlas,
        rnd: impl Into<RendererRef>,
    ) {
        let rnd: RendererRef = rnd.into();

        let old_col = rnd.draw_color_f32();
        rnd.set_draw_color_f32(Rgba::rgba(0., 0., 0., 0.5));
        let _ = rnd.fill_target();

        atlas.draw(rnd, self.prefix_id, TEXT_OFFSET);

        let mut r#where = PointF32::new(tbc, TEXT_OFFSET.y);
        let m_line_size_x = atlas.area(self.line_id).size.x;
        let mut crd = RectF32::xywh(m_line_size_x.min(m_wrap), m_outline.size.y, 0., 0.);

        while m_line_size_x - crd.pos.x > 0. {
            r#where.y += m_outline.size.y;
            crd.pos.x += m_wrap;
            crd.size.x = m_wrap.min(m_line_size_x - crd.pos.x);
            atlas.draw_part(rnd, self.line_id, crd, r#where);
        }

        if self.is_cursor_visible {
            rnd.set_draw_color_f32(Rgba::rgba(1., 1., 1., 0.5));
            let _ = rnd.fill_rect(m_outline);
        }

        if self.should_repaint {
            self.should_repaint = false;
            atlas.replace(self.line_id, rnd, self.make_line(font, placeholder));
        }

        rnd.set_draw_color_f32(old_col);
    }

    pub fn make_line(&self, font: impl Into<FontRef>, placeholder: usize) -> Surface {
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

    wrap_len: i32,
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

            wrap_len: wrap_len as _,
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
                self.wrap_len as _,
                self.outline,
                &self.font,
                self.placeholder_index as _,
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
        for evt in events {
            match evt {
                Event::KeyDown(k) => {
                    if k.key == SDLK_F1 {
                        self.switch(atlas, wnd)
                    }
                }
                Event::TextInput(e) => {
                    if let ConsoleState::Enabled(ac) = &mut self.state {
                        ac.process_str(unsafe { c_ptr_to_str(e.text) });
                    }
                }

                _ => (),
            }
        }
    }

    fn make_placeholder(&mut self) -> Surface {
        self.placeholder_index += 1;
        if self.placeholder_index as usize == PLACEHOLDERS.len() {
            self.placeholder_index = 0;
        }

        make_placeholder(&self.font, self.placeholder_index as _)
    }
}
