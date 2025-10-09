use std::{
    ffi::CStr,
    time::{Duration, Instant},
};

use halcyon::{
    color::Rgba,
    defs::SdlResult,
    rect::{Point, PointF32, PointI32, RectF32},
    renderer::RendererRef,
    surface::Surface,
    ttf::{Font, Text, TtfContext},
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

const PLACEHOLDERS: [&CStr; 39] = [
    c"[meow]",
    c"[redacted]",
    c"[your turn]",
    c"[womp womp]",
    c"[one big CVE]",
    c"[kevin's heart]",
    c"[lods of emone]",
    c"[be not afraid]",
    c"[see you again]",
    c"[forget me not]",
    c"[sudo deez nuts]",
    c"[openest source]",
    c"[at your service]",
    c"[with eye serene]",
    c"[is anyone there?]",
    c"[food for thought]",
    c"[made with Halcyon]",
    c"[49.0481N, 17.4838E]",
    c"[are you satisfied?]",
    c"[enter command here]",
    c"[running out of time]",
    c"[not actually random]",
    c"[watch?v=lo5cG0FhWro]",
    c"[not POSIX compliant]",
    c"[start typing, please]",
    c"[commands not included]",
    c"[segfaulting since 2021]",
    c"[waiting for user input]",
    c"[non-euclidean interface]",
    c"[who needs documentation]",
    c"[sudo pacman -S lyofetch]",
    c"[no man page here, sorry]",
    c"[ševalicious out tomorrow]",
    c"[licensed under the WTFPL]",
    c"[streets and sodium lights]",
    c"[quoth the raven, nevermore]",
    c"[docker? I barely know 'er!]",
    c"[rm -rf / --no-preserve-root]",
    c"[MSVC is the real final boss]",
];

const MAX_CHARS: u8 = 128;

const PREFIX_TEXT: &str = "raine1@Arctic~ %";
const PREFIX_TEXT_CSTR: &CStr = c"raine1@Arctic~ %";

const CURSOR_BLINK_TIME: Duration = Duration::from_millis(500);

const TEXT_OFFSET: PointF32 = Point::new(10., 10.);

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
        cursor.pos.x = tbc + (pos % MAX_CHARS as usize) as f32 * cursor.size.x;
        cursor.pos.y = TEXT_OFFSET.y + cursor.size.y * (pos / MAX_CHARS as usize) as f32;

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

        if self.field.text.len() > MAX_CHARS as usize {
            self.field.trim(MAX_CHARS as usize);
        }
    }

    /// Hand over a pressed key to this console's `Field`, it'll decide what to do next.
    /// Returns the rect that corresponds to the new outline.
    pub fn process_key(&mut self, tbc: f32, k: SDL_Keycode, cursor: RectF32) -> RectF32 {
        match k {
            e => {
                let op = self.field.process_key(e);

                if self.field.text.len() > MAX_CHARS as usize {
                    self.field.trim(MAX_CHARS as usize);
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

fn cast_point(pt: PointI32) -> PointF32 {
    Point::new(pt.x as _, pt.y as _)
}

impl Console {
    pub fn new(
        renderer: impl Into<RendererRef>,
        ttf: &TtfContext,
        epoch: Instant,
        base: ResourceLoader,
    ) -> SdlResult<Self> {
        let rs = cast_point(renderer.into().output_size());

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
        let outline = RectF32 {
            pos: Point::new(tex_begin_crd, TEXT_OFFSET.y),
            size: cast_point(Text::new(&font, " ")?.size().into()),
        };

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
                        .render_text_blended(PREFIX_TEXT_CSTR, Rgba::rgb(0, 255, 0))
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

    fn make_placeholder(&mut self) -> Surface {
        self.placeholder_index += 1;
        if self.placeholder_index as usize == PLACEHOLDERS.len() {
            self.placeholder_index = 0;
        }

        self.font
            .render_text_blended(
                PLACEHOLDERS[self.placeholder_index as usize],
                Rgba::rgb(0x80, 0x80, 0x80),
            )
            .unwrap()
    }
}
