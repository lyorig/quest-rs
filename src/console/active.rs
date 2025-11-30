use std::{rc::Rc, time::Duration};

use halcyon::{
    color::{Rgb, Rgba},
    guard::DrawColorGuard,
    rect::{PointF32, RectF32},
    renderer::RendererRef,
    surface::Surface,
};
use sdl3_sys::keycode::SDL_Keycode;

use crate::{
    atlas::{Atlas, AtlasId},
    command,
    console::{cache::CachedData, writer::ConsoleWriter},
    field::{Field, FieldAction},
    game::GameData,
};

const PLACEHOLDERS: [&str; 41] = [
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
    "[no more parties in L.A.]",
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

pub const TEXT_OFFSET: PointF32 = PointF32::new(10.0, 10.0);

pub fn make_placeholder(data: &mut CachedData) -> Surface {
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
    cursor_pos: f32,
    cursor_time: Duration,

    pub prefix_id: AtlasId,
    pub line_id: AtlasId,
    should_repaint: bool,

    writer: ConsoleWriter,
}

impl ActiveConsole {
    pub fn new(data: &CachedData, prefix_id: AtlasId, line_id: AtlasId) -> Self {
        Self {
            field: Field::new(),
            cursor_pos: data.input_x_origin,
            cursor_time: Duration::ZERO,
            prefix_id,
            line_id,
            should_repaint: true,
            writer: ConsoleWriter::new(),
        }
    }

    fn update_outline(&mut self, data: &mut CachedData) {
        self.cursor_pos = data.input_x_origin + self.field.cursor as f32 * data.glyph_size.x;
        self.cursor_time = Duration::ZERO;
    }

    pub fn update_delta(&mut self, delta: Duration) {
        self.cursor_time += delta;
    }

    pub fn process_str(&mut self, data: &mut CachedData, input: &str) {
        self.should_repaint = self.field.process_str(input);
        self.field.trim_check();
        self.update_outline(data);
    }

    /// Hands over a pressed key to this console's `Field`, which decides what to do next.
    /// Returns the rect that corresponds to the new outline.
    pub fn process_key(&mut self, data: &mut CachedData, k: SDL_Keycode) {
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

    pub fn process_command(&mut self, cd: &mut CachedData, gd: &mut GameData) {
        self.writer.clear();

        let mut args = self.field.text.split(' ');
        if let Some(name) = args.next() {
            match command::find(name) {
                Some(c) => c.execute(gd, &mut self.writer, args),
                None => self.writer.write(&format!("unknown command \"{name}\"")),
            }

            for line in self.writer.lines().filter(|l| !l.is_empty()) {
                // Four-star hotel, baby!
                cd.history
                    .push(match cd.history.iter().find(|h| ****h == *line) {
                        Some(a) => a.clone(),
                        None => {
                            cd.glyph_map.add(&mut gd.atlas, (&cd.font).into(), line);
                            Rc::new(line.into())
                        }
                    })
            }

            self.clear(cd);
        }
    }

    pub fn draw(&mut self, data: &mut CachedData, atlas: &mut Atlas, rnd: impl Into<RendererRef>) {
        let rnd: RendererRef = rnd.into();

        let dcl = DrawColorGuard::new(rnd, Rgba::new(Rgb::BLACK, 0.5));
        let _ = rnd.fill_target();

        let mut curr_draw = TEXT_OFFSET;

        for line in data.history.iter() {
            for glyph in line.chars() {
                if !glyph.is_whitespace() {
                    let id = data
                        .glyph_map
                        .id(glyph)
                        .expect("Drawn glyphs should be in the map");

                    atlas.draw(rnd, id, curr_draw);
                }

                curr_draw.x += data.glyph_size.x;
            }

            curr_draw.y += data.glyph_size.y;
            curr_draw.x = TEXT_OFFSET.x;
        }

        atlas.draw(rnd, self.prefix_id, curr_draw);
        atlas.draw(
            rnd,
            self.line_id,
            PointF32::new(data.input_x_origin, curr_draw.y),
        );

        dcl.set(Rgba::new(Rgb::WHITE, 0.5));

        if self.cursor_time.subsec_millis() < 500 {
            curr_draw.x = self.cursor_pos;
            let _ = rnd.fill_rect(RectF32::new(curr_draw, data.glyph_size));
        }

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

    /// Clear the `Field`, update the cursor,
    /// and signal for a repaint.
    pub fn clear(&mut self, data: &mut CachedData) {
        self.field.clear();
        self.update_outline(data);

        self.should_repaint = true;
    }
}
