use std::{fmt::Write, time::Duration};

use halcyon::{
    color::{Rgb, Rgba},
    rect::{PointF32, RectF32},
    renderer::Renderer,
    resource::{Ref, Resource},
    traits::ColorModF32,
};
use sdl3_sys::{events::SDL_MouseWheelEvent, keycode::SDL_Keycode, mouse::SDL_MouseWheelDirection};

use crate::{
    atlas::Atlas,
    chk,
    console::{CONSOLE_FONT, PREFIX_TEXT, cache::Cache, command, field::Field},
    font::store::FontStore,
    game::Game,
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
    pub fn new(data: &mut Cache) -> Self {
        Self {
            field: Field::new(),
            cursor_pos: data.input_x_origin,
            cursor_time: Duration::ZERO,
        }
    }

    fn update_outline(&mut self, cd: &mut Cache) {
        self.cursor_pos = cd.input_x_origin + self.field.cursor as f32 * cd.glyph_size.x;
        self.cursor_time = Duration::ZERO;
    }

    pub fn update_delta(&mut self, delta: Duration) {
        self.cursor_time += delta;
    }

    /// This is the only way that text gets added do the [`Field`].
    pub fn process_str(
        &mut self,
        cd: &mut Cache,
        fonts: &mut FontStore,
        atlas: &mut Atlas,
        input: &str,
    ) {
        self.field.process_str(input, fonts, atlas);
        self.field.trim_check();

        self.update_outline(cd);
    }

    /// Hands over a pressed key to this console's [`Field`], which decides what to do next.
    pub fn process_key(
        &mut self,
        cd: &mut Cache,
        fonts: &mut FontStore,
        atlas: &mut Atlas,
        k: SDL_Keycode,
    ) {
        let op = self.field.process_key(k, fonts, atlas);

        self.field.trim_check();

        if op {
            self.update_outline(cd);
        }
    }

    pub fn process_mouse(&mut self, rnd: Ref<Renderer>, cd: &mut Cache, m: SDL_MouseWheelEvent) {
        let val = if m.direction == SDL_MouseWheelDirection::NORMAL {
            m.integer_y
        } else {
            -m.integer_y
        };

        let new = cd.line.cast_signed() - val;
        cd.line = new.clamp(0, cd.total_lines() as _) as _;

        cd.clamp_line(rnd);
    }

    pub fn process_command(&mut self, res: &mut Game) {
        let mut args = self.field.text.split(' ');
        if let Some(name) = args.next()
            && !name.is_empty()
        {
            res.console_cache.writer.write_command(&self.field.text);

            match command::find(name) {
                Some(c) => c.execute(res, args),
                None => _ = writeln!(res.console_cache.writer, "sh: no such command: \"{name}\""),
            }

            let added = res.console_cache.writer.added_since_last_check();
            res.fonts.alloc(CONSOLE_FONT, added, &mut res.atlas);

            self.clear(
                &mut res.console_cache,
                res.renderer.as_ref(),
                &mut res.fonts,
                &mut res.atlas,
            );
        }
    }

    pub fn draw(
        &mut self,
        cd: &mut Cache,
        rnd: Ref<Renderer>,
        fonts: &mut FontStore,
        atlas: &mut Atlas,
    ) {
        let old = rnd.xchg_draw_color_f32(Rgb::BLACK.with_alpha(0.5));
        chk!(rnd.fill_target());

        let mut curr_draw = TEXT_OFFSET;

        for line in cd.writer.lines().skip(cd.line as _) {
            let Some(first) = line.bytes().next() else {
                continue;
            };

            if first == b'\0' {
                let old = atlas.tex().xchg_rgb_mod_f32(Rgb::GREEN);

                fonts.draw(
                    CONSOLE_FONT,
                    rnd,
                    atlas,
                    PREFIX_TEXT,
                    &mut curr_draw,
                    cd.glyph_size,
                );

                atlas.tex().set_rgb_mod_f32(old);

                curr_draw.x += cd.glyph_size.x;
                fonts.draw(
                    CONSOLE_FONT,
                    rnd,
                    atlas,
                    &line[1..],
                    &mut curr_draw,
                    cd.glyph_size,
                );
            } else {
                fonts.draw(
                    CONSOLE_FONT,
                    rnd,
                    atlas,
                    line,
                    &mut curr_draw,
                    cd.glyph_size,
                );
            }

            curr_draw.y += cd.glyph_size.y;
            curr_draw.x = TEXT_OFFSET.x;
        }

        self.draw_prompt(cd, rnd, fonts, atlas, curr_draw);

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
    pub fn clear(
        &mut self,
        cd: &mut Cache,
        rnd: Ref<Renderer>,
        fonts: &mut FontStore,
        atlas: &mut Atlas,
    ) {
        fonts.alloc(CONSOLE_FONT, &self.field.text, atlas);

        self.field.clear();
        self.update_outline(cd);
        cd.update_scroll_bar(rnd);
    }

    fn draw_prompt(
        &self,
        cd: &Cache,
        rnd: Ref<Renderer>,
        fonts: &mut FontStore,
        atlas: &mut Atlas,
        mut origin: PointF32,
    ) {
        let old = atlas.tex().xchg_rgb_mod_f32(Rgb::GREEN);

        fonts.draw(
            CONSOLE_FONT,
            rnd,
            atlas,
            PREFIX_TEXT,
            &mut origin,
            cd.glyph_size,
        );

        origin.x = cd.input_x_origin;

        let prompt = if self.field.text.is_empty() {
            // placeholder
            atlas.tex().set_rgb_mod_f32(Rgb::new(0.5, 0.5, 0.5));
            cd.current_placeholder()
        } else {
            // actual prompt
            atlas.tex().set_rgb_mod_f32(Rgb::WHITE);
            &self.field.text
        };

        fonts.draw(CONSOLE_FONT, rnd, atlas, prompt, &mut origin, cd.glyph_size);

        atlas.tex().set_rgb_mod_f32(old);
    }
}
