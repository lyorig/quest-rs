use std::time::Duration;

use halcyon::{
    color::{Rgb, Rgba},
    guard::{ColorModF32Guard, DrawColorGuard},
    rect::{PointF32, RectF32},
};
use sdl3_sys::keycode::SDL_Keycode;

use crate::{
    console::{CONSOLE_FONT, PREFIX_TEXT, cache::CachedData, command, field::Field},
    game::resources::GameResources,
};

pub const TEXT_OFFSET: PointF32 = PointF32::new(10.0, 10.0);

pub struct ActiveConsole {
    pub field: Field,

    /// Where the cursor is currently being drawn to.
    /// Only updated when [`ActiveConsole::update_outline()`] is called,
    /// which sets its location to correspond to the [`Field`] cursor.
    cursor_pos: f32,
    cursor_time: Duration,
}

impl ActiveConsole {
    pub fn new(data: &mut CachedData) -> Self {
        Self {
            field: Field::new(),
            cursor_pos: data.input_x_origin,
            cursor_time: Duration::ZERO,
        }
    }

    fn update_outline(&mut self, cd: &mut CachedData) {
        self.cursor_pos = cd.input_x_origin + self.field.cursor as f32 * cd.glyph_size.x;
        self.cursor_time = Duration::ZERO;
    }

    pub fn update_delta(&mut self, delta: Duration) {
        self.cursor_time += delta;
    }

    /// This is the only way that text gets added do the [`Field`].
    pub fn process_str(&mut self, cd: &mut CachedData, data: &mut GameResources, input: &str) {
        self.field.process_str(input, data);
        self.field.trim_check();

        self.update_outline(cd);
    }

    /// Hands over a pressed key to this console's [`Field`], which decides what to do next.
    pub fn process_key(&mut self, cd: &mut CachedData, data: &mut GameResources, k: SDL_Keycode) {
        let op = self.field.process_key(k, data);

        self.field.trim_check();

        if op {
            self.update_outline(cd);
        }
    }

    pub fn process_command(&mut self, cd: &mut CachedData, gd: &mut GameResources) {
        let mut args = self.field.text.split(' ');
        if let Some(name) = args.next() {
            match command::find(name) {
                Some(c) => c.execute(gd, &mut cd.writer, args),
                None => cd.writer.write(&format!("unknown command \"{name}\"")),
            }

            gd.font_alloc(CONSOLE_FONT, cd.writer.added_since_last_check());

            self.clear(cd, gd);
        }
    }

    pub fn draw(&mut self, cd: &mut CachedData, data: &GameResources) {
        let rnd = *data.renderer;
        let dcl = DrawColorGuard::new(rnd, Rgba::new(Rgb::BLACK, 0.5));
        let _ = rnd.fill_target();

        let mut curr_draw = TEXT_OFFSET;

        for line in cd.writer.lines() {
            data.font_draw(CONSOLE_FONT, line, &mut curr_draw, cd.glyph_size);

            curr_draw.y += cd.glyph_size.y;
            curr_draw.x = TEXT_OFFSET.x;
        }

        self.draw_prompt(cd, data, curr_draw);

        dcl.set(Rgba::new(Rgb::WHITE, 0.5));

        if self.cursor_time.subsec_millis() < 500 {
            curr_draw.x = self.cursor_pos;
            let _ = rnd.fill_rect(RectF32::new(curr_draw, cd.glyph_size));
        }
    }

    /// Clear the [`Field`], update the cursor,
    /// and signal for a repaint.
    pub fn clear(&mut self, cd: &mut CachedData, res: &mut GameResources) {
        res.font_free(CONSOLE_FONT, &self.field.text);
        self.field.clear();
        self.update_outline(cd);
    }

    fn draw_prompt(&self, cd: &CachedData, data: &GameResources, mut origin: PointF32) {
        let dcl = ColorModF32Guard::new(**data.atlas.texture.as_ref().unwrap(), Rgba::GREEN);

        data.font_draw(CONSOLE_FONT, PREFIX_TEXT, &mut origin, cd.glyph_size);

        origin.x = cd.input_x_origin;

        let prompt = if self.field.text.is_empty() {
            // placeholder
            dcl.set(Rgba::rgb(0.5, 0.5, 0.5));
            cd.current_placeholder()
        } else {
            // actual prompt
            dcl.set(Rgba::WHITE);
            &self.field.text
        };

        data.font_draw(CONSOLE_FONT, prompt, &mut origin, cd.glyph_size);
    }
}
