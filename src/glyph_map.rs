use std::collections::HashMap;

use halcyon::{color::Rgba, ttf::FontRef};

use crate::atlas::{Atlas, AtlasId};

struct GlyphData {
    pub count: usize,
    pub id: AtlasId,
}

/// Ref-counted glyphs!
pub struct GlyphMap {
    usage: HashMap<char, GlyphData>,
}

impl GlyphMap {
    pub fn new() -> Self {
        Self {
            usage: HashMap::new(),
        }
    }

    /// Add a string of glyphs into the map.
    /// This automatically filters out whitespace chars.
    pub fn add(&mut self, atlas: &mut Atlas, font: FontRef, text: &str) {
        for glyph in text.chars().filter(|c| !c.is_whitespace()) {
            match self.usage.get_mut(&glyph) {
                Some(u) => u.count += 1,
                None => {
                    let surf = font
                        .render_glyph_blended(glyph, Rgba::WHITE)
                        .expect("Glyph rendering really shouldn't be an issue");

                    self.usage.insert(
                        glyph,
                        GlyphData {
                            count: 1,
                            id: atlas.push(surf),
                        },
                    );
                }
            };
        }
    }

    pub fn id(&self, glyph: char) -> Option<AtlasId> {
        self.usage.get(&glyph).map(|f| f.id)
    }

    pub fn remove(&mut self, atlas: &mut Atlas, text: &str) {
        for glyph in text.chars() {
            if let Some(u) = self.usage.get_mut(&glyph) {
                u.count -= 1;
                if u.count == 0 {
                    atlas.remove(u.id);
                }
            };
        }
    }
}
