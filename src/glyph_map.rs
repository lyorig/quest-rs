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
                    let Ok(surf) = font.render_glyph_blended(glyph, Rgba::WHITE) else {
                        panic!("Failed to render glyph {glyph}");
                    };

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

    pub fn id(&self, glyph: char) -> AtlasId {
        match self.usage.get(&glyph).map(|f| f.id) {
            Some(c) => c,
            None => panic!("Glyph \"{glyph}\" not present"),
        }
    }

    pub fn remove_str(&mut self, atlas: &mut Atlas, text: &str) {
        for glyph in text.chars() {
            self.remove(atlas, glyph);
        }
    }

    pub fn remove(&mut self, atlas: &mut Atlas, glyph: char) {
        if glyph.is_whitespace() {
            return;
        }

        match self.usage.get_mut(&glyph) {
            Some(u) => {
                u.count -= 1;
                if u.count == 0 {
                    atlas.remove(u.id);
                    self.usage.remove(&glyph);
                }
            }
            None => panic!("Cannot remove unused character {glyph}"),
        };
    }
}
