use std::collections::HashMap;

use halcyon::{color::Rgba, rect::PointF32, ttf::FontRef};

use crate::{
    atlas::{Atlas, AtlasId},
    game::GameData,
};

struct GlyphData {
    refcount: u32,
    id: AtlasId,
}

impl GlyphData {
    fn new(refcount: u32, id: AtlasId) -> Self {
        Self { refcount, id }
    }
}

/// Manages ref-counts for glyphs contained in an [`Atlas`].
pub struct GlyphMap {
    usage: HashMap<char, GlyphData>,
}

impl GlyphMap {
    pub fn new() -> Self {
        Self {
            usage: HashMap::new(),
        }
    }

    pub fn push(&mut self, atlas: &mut Atlas, font: FontRef, glyph: char) {
        match self.usage.get_mut(&glyph) {
            Some(k) => {
                k.refcount += 1;
            }
            None => {
                let surf = font.render_glyph_blended(glyph, Rgba::WHITE).unwrap();
                let id = atlas.push(surf);

                self.usage.insert(glyph, GlyphData::new(1, id));
            }
        }
    }

    pub fn push_str(&mut self, atlas: &mut Atlas, font: FontRef, text: &str) {
        for c in text.chars() {
            self.push(atlas, font, c);
        }
    }

    pub fn pop(&mut self, atlas: &mut Atlas, glyph: char) {
        let Some(data) = self.usage.get_mut(&glyph) else {
            panic!("[Atlas] Popping non-existent glyph '{glyph}'");
        };

        data.refcount -= 1;

        if data.refcount == 0 {
            atlas.remove(data.id);
        }
    }

    pub fn pop_str(&mut self, atlas: &mut Atlas, text: &str) {
        for c in text.chars() {
            self.pop(atlas, c);
        }
    }

    /// Retrieve an [`AtlasId`] for a glyph.
    fn id(&self, glyph: char) -> Option<AtlasId> {
        self.usage.get(&glyph).map(|gd| gd.id)
    }

    /// Convenience method for drawing a string to the screen.
    /// Panics if any character in `text` isn't available in glyph form in `atlas`.
    pub fn draw(&self, text: &str, data: &GameData, origin: &mut PointF32, glyph_width: f32) {
        let rnd = *data.renderer;
        for glyph in text.chars() {
            if !glyph.is_whitespace() {
                let Some(id) = self.id(glyph) else {
                    panic!("[Atlas] Cannot draw unavailable glyph '{glyph}'")
                };
                data.atlas.draw(rnd, id, *origin);
            }

            origin.x += glyph_width;
        }
    }
}
