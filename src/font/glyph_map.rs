use std::num::NonZeroU32;

use halcyon::{
    color::Rgba,
    rect::{PointF32, RectF32},
    traits::{Ref, Resource},
    ttf::Font,
};

use crate::{
    atlas::{Atlas, AtlasId},
    game::resources::GameResources,
};

#[derive(Clone, Copy)]
struct GlyphData {
    refcount: NonZeroU32,
    id: AtlasId,
}

impl GlyphData {
    fn can_delete(&self) -> bool {
        self.refcount == NonZeroU32::MIN
    }
}

#[derive(Clone, Copy)]
enum GlyphEntry {
    Free,
    Allocated(GlyphData),
}

/// 33 ('!') - 126 ('~')
const NUM_GRAPHIC_ASCII_CHARS: usize = 94;

/// Manages ref-counts for glyphs contained in an [`Atlas`].
pub(crate) struct GlyphMap {
    /// Graphical ASCII characters, minus space (0x20) and delete (0x7F).
    usage: [GlyphEntry; NUM_GRAPHIC_ASCII_CHARS],
}

impl GlyphMap {
    pub fn new() -> Self {
        Self {
            usage: [GlyphEntry::Free; NUM_GRAPHIC_ASCII_CHARS],
        }
    }

    fn retain_glyph(&mut self, atlas: &mut Atlas, font: Ref<Font>, glyph: char) {
        Self::assert_printable(glyph);

        let entry = &mut self.usage[Self::char_index(glyph)];

        match entry {
            GlyphEntry::Allocated(gd) => {
                gd.refcount = gd
                    .refcount
                    .checked_add(1)
                    .expect("[GlyphMap] Entry refcount overflow")
            }
            GlyphEntry::Free => {
                let surf = font
                    .render_glyph_shaded(glyph, Rgba::WHITE, Rgba::BLACK)
                    .unwrap();

                *entry = GlyphEntry::Allocated(GlyphData {
                    // NOTE: NonZeroU32::MIN is used as a sentinel value, noting that this glyph
                    // is available for deallocation. This is a somewhat hacky way to go about it,
                    // but it leads a smaller struct size (8 vs. 12 bytes).
                    refcount: unsafe { NonZeroU32::new_unchecked(NonZeroU32::MIN.get() + 1) },
                    id: atlas.push(surf),
                });
            }
        }
    }

    pub fn retain(&mut self, atlas: &mut Atlas, font: Ref<Font>, text: &str) {
        for c in text.chars().filter(char::is_ascii_graphic) {
            self.retain_glyph(atlas, font, c);
        }
    }

    fn release_glyph(&mut self, glyph: char) {
        let GlyphEntry::Allocated(data) = &mut self.usage[Self::char_index(glyph)] else {
            panic!("[GlyphMap] Popping unallocated glyph '{glyph}'");
        };

        assert!(
            data.refcount != NonZeroU32::MIN,
            "[GlyphMap] Popping scheduled-for-deletion glyph '{glyph}"
        );

        // SAFETY: The contained value is guaranteed to be non-minimal.
        data.refcount = unsafe { NonZeroU32::new_unchecked(data.refcount.get().wrapping_sub(1)) };
    }

    pub fn release(&mut self, text: &str) {
        for c in text.chars().filter(char::is_ascii_graphic) {
            self.release_glyph(c);
        }
    }

    /// Perform garbage collection, i.e. remove all unreferenced glyphs.
    /// It's up to you when you call this; a glyph may be requested while
    /// unreferenced, which is basically free. If this method is instead called
    /// beforehand, it's removed from the [`Atlas`] and all the work required
    /// for re-insertion must be performed.
    pub fn gc(&mut self, atlas: &mut Atlas) {
        for data in &mut self.usage {
            if let GlyphEntry::Allocated(gd) = data
                && gd.can_delete()
            {
                atlas.remove(gd.id);
                *data = GlyphEntry::Free;
            }
        }
    }

    /// Retrieve an [`AtlasId`] for a glyph.
    /// This succeeds even if a glyph is scheduled for deletion.
    fn id(&self, glyph: char) -> Option<AtlasId> {
        match self.usage[Self::char_index(glyph)] {
            GlyphEntry::Free => None,
            GlyphEntry::Allocated(gd) => Some(gd.id),
        }
    }

    /// Convenience method for drawing a string to the screen.
    /// Panics if any character in `text` isn't available in glyph form in `atlas`.
    pub fn draw(
        &self,
        text: &str,
        res: &GameResources,
        origin: &mut PointF32,
        glyph_size: PointF32,
    ) {
        for glyph in text.chars() {
            if !glyph.is_whitespace() {
                let Some(id) = self.id(glyph) else {
                    panic!("[GlyphMap] Cannot draw unavailable glyph '{glyph}'")
                };

                res.atlas
                    .draw_to(res.renderer.as_ref(), id, RectF32::new(*origin, glyph_size));
            }

            origin.x += glyph_size.x;
        }
    }

    fn char_index(c: char) -> usize {
        Self::assert_printable(c);

        (c as usize) - ('!' as usize)
    }

    fn assert_printable(c: char) {
        assert!(
            c.is_ascii_graphic(),
            "[GlyphMap] Pushing unsupported value (ASCII 0x{:x})",
            c as u32
        );
    }
}
