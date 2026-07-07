use std::{mem::MaybeUninit, num::NonZeroU32};

use halcyon::{
    color::Rgba,
    rect::{PointF32, RectF32},
    traits::{Ref, Resource},
    ttf::Font,
};

use crate::{
    atlas::{Atlas, AtlasId},
    game::resources::Resources,
};

/// 33 ('!') - 126 ('~')
const GRAPHIC_ASCII_FIRST: char = '!';
const GRAPHIC_ASCII_LAST: char = '~';
const NUM_GRAPHIC_ASCII_CHARS: usize = 94;

pub trait GlyphProvider {
    fn new(atlas: &mut Atlas, font: Ref<Font>) -> Self;

    fn retain_glyph(&mut self, atlas: &mut Atlas, font: Ref<Font>, glyph: char);
    fn release_glyph(&mut self, glyph: char);

    /// Retrieve an [`AtlasId`] for a glyph.
    fn id(&self, glyph: char) -> Option<AtlasId>;

    /// Convenience method for drawing a string to the screen.
    /// Panics if any character in `text` isn't available in glyph form in `atlas`.
    fn draw(&self, text: &str, res: &Resources, origin: &mut PointF32, glyph_size: PointF32) {
        for glyph in text.chars() {
            if !glyph.is_whitespace() {
                let Some(id) = self.id(glyph) else {
                    panic!("[GlyphMap] Cannot draw unavailable glyph '{glyph}' from {text:?}")
                };

                res.atlas
                    .draw_to(res.renderer.as_ref(), id, RectF32::new(*origin, glyph_size));
            }

            origin.x += glyph_size.x;
        }
    }

    fn retain(&mut self, atlas: &mut Atlas, font: Ref<Font>, text: &str) {
        text.chars()
            .filter(char::is_ascii_graphic)
            .for_each(|c| self.retain_glyph(atlas, font, c));
    }

    fn release(&mut self, text: &str) {
        text.chars()
            .filter(char::is_ascii_graphic)
            .for_each(|c| self.release_glyph(c));
    }

    fn char_index(c: char) -> usize {
        assert!(c.is_ascii_graphic());
        (c as usize) - (GRAPHIC_ASCII_FIRST as usize)
    }
}

pub struct LazyGlyphMap {
    /// Graphical ASCII characters, minus space (0x20) and delete (0x7F).
    usage: [Option<AtlasId>; NUM_GRAPHIC_ASCII_CHARS],
}

impl GlyphProvider for LazyGlyphMap {
    fn new(_atlas: &mut Atlas, _font: Ref<Font>) -> Self {
        Self { usage: [None; _] }
    }

    fn retain_glyph(&mut self, atlas: &mut Atlas, font: Ref<Font>, glyph: char) {
        let entry = &mut self.usage[Self::char_index(glyph)];
        if entry.is_none() {
            let surf = font.render_glyph_blended(glyph, Rgba::WHITE).unwrap();
            let id = atlas.push(surf);
            *entry = Some(id);
        }
    }

    /// No-op, glyphs remain loaded.
    fn release_glyph(&mut self, _glyph: char) {}

    fn id(&self, glyph: char) -> Option<AtlasId> {
        self.usage[Self::char_index(glyph)]
    }
}

pub struct PreloadedGlyphMap {
    glyphs: [AtlasId; NUM_GRAPHIC_ASCII_CHARS],
}

impl GlyphProvider for PreloadedGlyphMap {
    fn new(atlas: &mut crate::atlas::Atlas, font: Ref<Font>) -> Self {
        let mut array = [MaybeUninit::<AtlasId>::uninit(); NUM_GRAPHIC_ASCII_CHARS];
        let range = GRAPHIC_ASCII_FIRST..=GRAPHIC_ASCII_LAST;
        for (c, a) in range.zip(array.iter_mut()) {
            let surf = font.render_glyph_blended(c, Rgba::WHITE).unwrap();
            let id = atlas.push(surf);
            a.write(id);
        }

        Self {
            glyphs: array.map(|id| unsafe { id.assume_init() }),
        }
    }

    /// No-op, glyph is already retained.
    fn retain_glyph(&mut self, _atlas: &mut crate::atlas::Atlas, _font: Ref<Font>, _glyph: char) {}

    /// No-op, glyphs remain retained.
    fn release_glyph(&mut self, _glyph: char) {}

    fn id(&self, glyph: char) -> Option<AtlasId> {
        let id = self.glyphs[glyph as usize - GRAPHIC_ASCII_FIRST as usize];
        Some(id)
    }
}

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

pub struct RefcountGlyphMap {
    /// Graphical ASCII characters, minus space (0x20) and delete (0x7F).
    usage: [Option<GlyphData>; NUM_GRAPHIC_ASCII_CHARS],
}

impl GlyphProvider for RefcountGlyphMap {
    fn new(_atlas: &mut Atlas, _font: Ref<Font>) -> Self {
        Self {
            usage: [None; NUM_GRAPHIC_ASCII_CHARS],
        }
    }

    fn retain_glyph(&mut self, atlas: &mut Atlas, font: Ref<Font>, glyph: char) {
        Self::assert_printable(glyph);

        let entry = &mut self.usage[Self::char_index(glyph)];

        match entry {
            Some(gd) => {
                gd.refcount = gd
                    .refcount
                    .checked_add(1)
                    .expect("[GlyphMap] Entry refcount overflow")
            }
            None => {
                let surf = font
                    .render_glyph_shaded(glyph, Rgba::WHITE, Rgba::BLACK)
                    .unwrap();

                *entry = Some(GlyphData {
                    // NOTE: NonZeroU32::MIN is used as a sentinel value, noting that this glyph
                    // is available for deallocation. This is a somewhat hacky way to go about it,
                    // but it leads a smaller struct size (8 vs. 12 bytes).
                    refcount: unsafe { NonZeroU32::new_unchecked(NonZeroU32::MIN.get() + 1) },
                    id: atlas.push(surf),
                });
            }
        }
    }

    fn release_glyph(&mut self, glyph: char) {
        let Some(data) = &mut self.usage[Self::char_index(glyph)] else {
            panic!("[GlyphMap] Popping unallocated glyph '{glyph}'");
        };

        assert!(
            data.refcount != NonZeroU32::MIN,
            "[GlyphMap] Popping scheduled-for-deletion glyph '{glyph}'"
        );

        // SAFETY: The contained value is guaranteed to be non-minimal.
        data.refcount = unsafe { NonZeroU32::new_unchecked(data.refcount.get().wrapping_sub(1)) };
    }

    /// Retrieve an [`AtlasId`] for a glyph.
    /// This succeeds even if a glyph is scheduled for deletion.
    fn id(&self, glyph: char) -> Option<AtlasId> {
        self.usage[Self::char_index(glyph)].map(|gd| gd.id)
    }
}

impl RefcountGlyphMap {
    /// Perform garbage collection, i.e. remove all unreferenced glyphs.
    /// It's up to you when you call this; a glyph may be requested while
    /// unreferenced, which is basically free. If this method is instead called
    /// beforehand, it's removed from the [`Atlas`] and all the work required
    /// for re-insertion must be performed.
    pub fn gc(&mut self, atlas: &mut Atlas) -> usize {
        let mut count = 0;
        for data in &mut self.usage {
            if let Some(gd) = data
                && gd.can_delete()
            {
                atlas.remove(gd.id);
                *data = None;
                count += 1;
            }
        }

        count
    }

    /// Convenience method for drawing a string to the screen.
    /// Panics if any character in `text` isn't available in glyph form in `atlas`.
    pub fn draw(&self, text: &str, res: &Resources, origin: &mut PointF32, glyph_size: PointF32) {
        for glyph in text.chars() {
            if !glyph.is_whitespace() {
                let Some(id) = self.id(glyph) else {
                    panic!("[GlyphMap] Cannot draw unavailable glyph '{glyph}' from {text:?}")
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
