use halcyon::{color::Rgba, ttf::FontRef};

use crate::atlas::Atlas;

struct GlyphMap {}

impl GlyphMap {
    pub fn new() -> Self {
        Self {}
    }

    pub fn add(atlas: &mut Atlas, font: FontRef, text: &str) {
        for glyph in text.chars() {
            atlas.push(
                font.render_glyph_solid(glyph, Rgba::BLACK)
                    .expect("Glyph rendering really shouldn't be an issue"),
            );
        }
    }
}
