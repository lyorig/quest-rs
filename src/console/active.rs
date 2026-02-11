use std::time::Duration;

use halcyon::{
    color::{Rgb, Rgba},
    guard::{ColorModF32Guard, DrawColorGuard},
    rect::{PointF32, RectF32},
};
use sdl3_sys::keycode::SDL_Keycode;

use crate::{
    console::{PREFIX_TEXT, cache::CachedData, command, field::Field, writer::ConsoleWriter},
    game::GameData,
};

pub const TEXT_OFFSET: PointF32 = PointF32::new(10.0, 10.0);

pub struct ActiveConsole {
    pub field: Field,

    /// Where the cursor is currently being drawn to.
    /// Only updated when `self.update_outline()` is called,
    /// which sets its location to correspond to the [`Field`] cursor.
    cursor_pos: f32,
    cursor_time: Duration,

    writer: ConsoleWriter,
}

impl ActiveConsole {
    pub fn new(data: &mut CachedData) -> Self {
        Self {
            field: Field::new(),
            cursor_pos: data.input_x_origin,
            cursor_time: Duration::ZERO,
            writer: ConsoleWriter::new(),
        }
    }

    fn update_outline(&mut self, cd: &mut CachedData, data: &GameData) {
        self.cursor_pos =
            cd.input_x_origin + self.field.cursor as f32 * data.font_ubuntu.glyph_size.x;
        self.cursor_time = Duration::ZERO;
    }

    pub fn update_delta(&mut self, delta: Duration) {
        self.cursor_time += delta;
    }

    /// This is the only way that text gets added do the [`Field`].
    pub fn process_str(&mut self, cd: &mut CachedData, data: &GameData, input: &str) {
        self.field.process_str(input);
        self.field.trim_check();

        self.update_outline(cd, data);
    }

    /// Hands over a pressed key to this console's [`Field`], which decides what to do next.
    pub fn process_key(&mut self, cd: &mut CachedData, data: &GameData, k: SDL_Keycode) {
        let op = self.field.process_key(k);

        self.field.trim_check();

        if op {
            self.update_outline(cd, data);
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

            self.clear(cd, gd);
        }
    }

    pub fn draw(&mut self, cd: &mut CachedData, data: &GameData) {
        let rnd = *data.renderer;
        let dcl = DrawColorGuard::new(rnd, Rgba::new(Rgb::BLACK, 0.5));
        let _ = rnd.fill_target();

        let mut curr_draw = TEXT_OFFSET;

        for line in cd.history.iter() {
            data.font_ubuntu.draw(line, data, &mut curr_draw);

            curr_draw.y += data.font_ubuntu.glyph_size.y;
            curr_draw.x = TEXT_OFFSET.x;
        }

        self.draw_prompt(cd, data, curr_draw);

        dcl.set(Rgba::new(Rgb::WHITE, 0.5));

        if self.cursor_time.subsec_millis() < 500 {
            curr_draw.x = self.cursor_pos;
            let _ = rnd.fill_rect(RectF32::new(curr_draw, data.font_ubuntu.glyph_size));
        }
    }

    /// Clear the `Field`, update the cursor,
    /// and signal for a repaint.
    pub fn clear(&mut self, cd: &mut CachedData, data: &GameData) {
        self.field.clear();
        self.update_outline(cd, data);
    }

    pub fn should_free_placeholder(&self) -> bool {
        self.field.text.is_empty()
    }

    fn draw_prompt(&self, cd: &CachedData, data: &GameData, mut origin: PointF32) {
        let dcl = ColorModF32Guard::new(**data.atlas.texture.as_ref().unwrap(), Rgba::GREEN);

        data.font_ubuntu.draw(PREFIX_TEXT, data, &mut origin);

        origin.x += data.font_ubuntu.glyph_size.x;

        let prompt = if self.field.text.is_empty() {
            // placeholder
            dcl.set(Rgba::rgb(0.5, 0.5, 0.5));
            cd.current_placeholder()
        } else {
            // actual prompt
            dcl.set(Rgba::WHITE);
            &self.field.text
        };

        data.font_ubuntu.draw(prompt, data, &mut origin);
    }
}
