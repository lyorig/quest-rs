use std::time::Duration;

use halcyon::{
    color::{Rgb, Rgba},
    rect::{PointF32, RectF32},
    traits::{ColorModF32, Resource},
};
use sdl3_sys::{events::SDL_MouseWheelEvent, keycode::SDL_Keycode, mouse::SDL_MouseWheelDirection};

use crate::{
    chk,
    console::{CONSOLE_FONT, PREFIX_TEXT, cache::CachedData, command, field::Field},
    game::resources::Resources,
};

pub const TEXT_OFFSET: PointF32 = PointF32::new(10.0, 10.0);

pub struct Inner {
    pub field: Field,

    /// Where the cursor is currently being drawn to.
    /// Only updated when [`Self::update_outline`] is called,
    /// which sets its location to correspond to the [`Field`] cursor.
    cursor_pos: f32,
    cursor_time: Duration,
}

impl Inner {
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
    pub fn process_str(&mut self, cd: &mut CachedData, data: &mut Resources, input: &str) {
        self.field.process_str(input, data);
        self.field.trim_check();

        self.update_outline(cd);
    }

    /// Hands over a pressed key to this console's [`Field`], which decides what to do next.
    pub fn process_key(&mut self, cd: &mut CachedData, data: &mut Resources, k: SDL_Keycode) {
        let op = self.field.process_key(k, data);

        self.field.trim_check();

        if op {
            self.update_outline(cd);
        }
    }

    pub fn process_mouse(
        &mut self,
        res: &mut Resources,
        cd: &mut CachedData,
        m: SDL_MouseWheelEvent,
    ) {
        let val = if m.direction == SDL_MouseWheelDirection::NORMAL {
            m.integer_y
        } else {
            -m.integer_y
        };

        let new = cd.line.cast_signed() - val;
        cd.line = new.clamp(0, cd.total_lines() as _) as _;

        cd.clamp_line(res);
    }

    pub fn process_command(&mut self, cd: &mut CachedData, gd: &mut Resources) {
        let mut args = self.field.text.split(' ');
        if let Some(name) = args.next()
            && !name.is_empty()
        {
            cd.writer.write_command(&self.field.text);

            match command::find(name) {
                Some(c) => c.execute(gd, cd, args),
                None => {
                    let msg = format!("unknown command \"{name}\"");
                    cd.writer.writeln(&msg)
                }
            }

            let added = cd.writer.added_since_last_check();
            gd.font_alloc(CONSOLE_FONT, added);

            self.clear(cd, gd);
        }
    }

    pub fn draw(&mut self, cd: &mut CachedData, data: &Resources) {
        let rnd = data.renderer.as_ref();
        let old = rnd.xchg_draw_color_f32(Rgb::BLACK.with_alpha(0.5));
        chk!(rnd.fill_target());

        let mut curr_draw = TEXT_OFFSET;

        for line in cd.writer.lines().skip(cd.line as _) {
            let Some(first) = line.bytes().next() else {
                continue;
            };

            if first == b'\0' {
                let texref = data.atlas.tex();
                let old = texref.xchg_rgb_mod_f32(Rgb::GREEN);

                data.font_draw(CONSOLE_FONT, PREFIX_TEXT, &mut curr_draw, cd.glyph_size);
                texref.set_rgb_mod_f32(old);

                curr_draw.x += cd.glyph_size.x;
                data.font_draw(CONSOLE_FONT, &line[1..], &mut curr_draw, cd.glyph_size);
            } else {
                data.font_draw(CONSOLE_FONT, line, &mut curr_draw, cd.glyph_size);
            }

            curr_draw.y += cd.glyph_size.y;
            curr_draw.x = TEXT_OFFSET.x;
        }

        self.draw_prompt(cd, data, curr_draw);

        rnd.set_draw_color_f32(Rgb::WHITE.with_alpha(0.5));

        if self.cursor_time.subsec_millis() < 500 {
            curr_draw.x = self.cursor_pos;
            chk!(rnd.fill_rect(RectF32::new(curr_draw, cd.glyph_size)));
        }

        rnd.set_draw_color_f32(Rgba::new(0.5, 0.5, 0.5, 0.8));
        chk!(rnd.fill_rect(cd.scroll_bar));

        rnd.set_draw_color_f32(old);
    }

    /// Clear the [`Field`], update the cursor,
    /// and signal for a repaint.
    pub fn clear(&mut self, cd: &mut CachedData, res: &mut Resources) {
        res.font_free(CONSOLE_FONT, &self.field.text);

        self.field.clear();
        self.update_outline(cd);
        cd.update_scroll_bar(res);
    }

    fn draw_prompt(&self, cd: &CachedData, data: &Resources, mut origin: PointF32) {
        let tex = data.atlas.tex();
        let old = tex.xchg_rgb_mod_f32(Rgb::GREEN);

        data.font_draw(CONSOLE_FONT, PREFIX_TEXT, &mut origin, cd.glyph_size);

        origin.x = cd.input_x_origin;

        let prompt = if self.field.text.is_empty() {
            // placeholder
            tex.set_rgb_mod_f32(Rgb::new(0.5, 0.5, 0.5));
            cd.current_placeholder()
        } else {
            // actual prompt
            tex.set_rgb_mod_f32(Rgb::WHITE);
            &self.field.text
        };

        data.font_draw(CONSOLE_FONT, prompt, &mut origin, cd.glyph_size);

        tex.set_rgb_mod_f32(old);
    }
}
