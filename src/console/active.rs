use std::{rc::Rc, time::Duration};

use halcyon::{
    color::{Rgb, Rgba},
    guard::{ColorModF32Guard, DrawColorGuard},
    rect::{PointF32, RectF32},
    renderer::RendererRef,
};
use sdl3_sys::keycode::SDL_Keycode;

use crate::{
    atlas::Atlas,
    console::{PREFIX_TEXT, cache::CachedData, command, field::Field, writer::ConsoleWriter},
    game::GameData,
};

pub const TEXT_OFFSET: PointF32 = PointF32::new(10.0, 10.0);

pub struct ActiveConsole {
    pub field: Field,

    /// Where the cursor is currently being drawn to.
    /// Only updated when `self.update_outline()` is called,
    /// which sets its location to correspond to the `Field` cursor.
    cursor_pos: f32,
    cursor_time: Duration,

    writer: ConsoleWriter,
}

impl ActiveConsole {
    pub fn new(data: &CachedData) -> Self {
        Self {
            field: Field::new(),
            cursor_pos: data.input_x_origin,
            cursor_time: Duration::ZERO,
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

    pub fn process_str(&mut self, atlas: &mut Atlas, data: &mut CachedData, input: &str) {
        self.field.process_str(input);
        self.field.trim_check();

        self.update_outline(data);
        data.glyph_map.add(atlas, *data.font, &self.field.text);
    }

    /// Hands over a pressed key to this console's `Field`, which decides what to do next.
    /// Returns the rect that corresponds to the new outline.
    pub fn process_key(&mut self, atlas: &mut Atlas, data: &mut CachedData, k: SDL_Keycode) {
        let op = self.field.process_key(k, atlas, &mut data.glyph_map);

        self.field.trim_check();

        // TODO: Convert to a "nicer" (not visually!) block with fallthroughs.
        if op {
            self.update_outline(data);
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
                    let id = data.glyph_map.id(glyph);

                    atlas.draw(rnd, id, curr_draw);
                }

                curr_draw.x += data.glyph_size.x;
            }

            curr_draw.y += data.glyph_size.y;
            curr_draw.x = TEXT_OFFSET.x;
        }

        self.draw_prompt(rnd, atlas, data, curr_draw);

        dcl.set(Rgba::new(Rgb::WHITE, 0.5));

        if self.cursor_time.subsec_millis() < 500 {
            curr_draw.x = self.cursor_pos;
            let _ = rnd.fill_rect(RectF32::new(curr_draw, data.glyph_size));
        }
    }

    /// Clear the `Field`, update the cursor,
    /// and signal for a repaint.
    pub fn clear(&mut self, data: &mut CachedData) {
        self.field.clear();
        self.update_outline(data);
    }

    fn draw_prompt(
        &self,
        rnd: RendererRef,
        atlas: &mut Atlas,
        cd: &CachedData,
        mut origin: PointF32,
    ) {
        let dcl = ColorModF32Guard::new(**atlas.texture.as_ref().unwrap(), Rgba::GREEN);

        // raine1@Arctic %~
        for glyph in PREFIX_TEXT.chars() {
            if !glyph.is_whitespace() {
                atlas.draw(rnd, cd.glyph_map.id(glyph), origin);
            }

            origin.x += cd.glyph_size.x;
        }

        origin.x += cd.glyph_size.x;

        let prompt = if self.field.text.is_empty() {
            // placeholder
            dcl.set(Rgba::rgb(0.5, 0.5, 0.5));
            cd.current_placeholder()
        } else {
            // actual prompt
            dcl.set(Rgba::WHITE);
            &self.field.text
        };

        // prompt
        for glyph in prompt.chars() {
            if !glyph.is_whitespace() {
                atlas.draw(rnd, cd.glyph_map.id(glyph), origin);
            }

            origin.x += cd.glyph_size.x;
        }
    }
}
