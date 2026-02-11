use std::ffi::CStr;

use halcyon::{
    rect::PointF32,
    ttf::{Font as HalFont, Text, TtfContext},
};

use crate::{atlas::Atlas, game::GameData, glyph_map::GlyphMap};

pub struct Font<'a> {
    font: HalFont<'a>,
    pub glyph_size: PointF32,
    map: GlyphMap,
}

impl Font<'_> {
    pub fn new<'a>(ttf: &'a TtfContext, font_path: &CStr, size: f32) -> Font<'a> {
        let font = HalFont::new(ttf, font_path, size).expect("Cannot open font");
        let glyph_size = Text::new(&font, "X").unwrap().size().into();
        let map = GlyphMap::new();

        Font {
            font,
            glyph_size,
            map,
        }
    }

    pub fn alloc(&mut self, text: &str, atlas: &mut Atlas) {
        self.map.push_str(atlas, *self.font, text);
    }

    pub fn free(&mut self, text: &str, atlas: &mut Atlas) {
        self.map.pop_str(atlas, text);
    }

    pub fn draw(&self, text: &str, data: &GameData, origin: &mut PointF32) {
        self.map.draw(text, data, origin, self.glyph_size.x);
    }
}
