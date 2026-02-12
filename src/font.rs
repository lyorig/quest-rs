use std::{
    ffi::CStr,
    num::{NonZero, NonZeroU32},
};

use halcyon::{
    color::Rgba,
    rect::PointF32,
    ttf::{Font as HalFont, FontRef, Text, TtfContext},
};

use crate::{
    atlas::{Atlas, AtlasId},
    game::GameData,
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

/// 33 ('!') 126 ('~')
const NUM_GRAPHIC_ASCII_CHARS: usize = 94;

/// Manages ref-counts for glyphs contained in an [`Atlas`].
struct GlyphMap {
    /// Graphical ASCII characters, minus space (0x20) and delete (0x7F).
    usage: [GlyphEntry; NUM_GRAPHIC_ASCII_CHARS],
}

impl GlyphMap {
    fn new() -> Self {
        Self {
            usage: [GlyphEntry::Free; NUM_GRAPHIC_ASCII_CHARS],
        }
    }

    fn push(&mut self, atlas: &mut Atlas, font: FontRef, glyph: char) {
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
                let surf = font.render_glyph_blended(glyph, Rgba::WHITE).unwrap();
                *entry = GlyphEntry::Allocated(GlyphData {
                    // NOTE: NonZeroU32::MIN is used as a sentinel value, noting that this glyph
                    // is available for deallocation. This is a somewhat hacky way to go about it,
                    // but it leads a smaller struct size (8 vs. 12 bytes).
                    refcount: unsafe { NonZero::new_unchecked(NonZeroU32::MIN.get() + 1) },
                    id: atlas.push(surf),
                });
            }
        }
    }

    fn push_str(&mut self, atlas: &mut Atlas, font: FontRef, text: &str) {
        for c in text.chars().filter(|c| Self::is_printable(*c)) {
            self.push(atlas, font, c);
        }
    }

    fn pop(&mut self, glyph: char) {
        let GlyphEntry::Allocated(data) = &mut self.usage[Self::char_index(glyph)] else {
            panic!("[GlyphMap] Popping unallocated glyph '{glyph}'");
        };

        assert!(
            data.refcount != NonZeroU32::MIN,
            "[GlyphMap] Popping scheduled-for-deletion glyph '{glyph}"
        );

        // SAFETY: The contained value is guaranteed to be non-minimal.
        data.refcount = unsafe { NonZeroU32::new_unchecked(data.refcount.get() - 1) };
    }

    fn pop_str(&mut self, text: &str) {
        for c in text.chars().filter(|c| Self::is_printable(*c)) {
            self.pop(c);
        }
    }

    /// Perform garbage collection, i.e. remove all unreferenced glyphs.
    /// It's up to you when you call this; a glyph may be requested while
    /// unreferenced, which is basically free. If this method is instead called
    /// beforehand, it's removed from the [`Atlas`] and all the work required
    /// for re-insertion must be performed.
    fn gc(&mut self, atlas: &mut Atlas) {
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
    fn draw(&self, text: &str, data: &GameData, origin: &mut PointF32, glyph_width: f32) {
        let rnd = *data.renderer;
        for glyph in text.chars() {
            if !glyph.is_whitespace() {
                let Some(id) = self.id(glyph) else {
                    panic!("[GlyphMap] Cannot draw unavailable glyph '{glyph}'")
                };

                data.atlas.draw(rnd, id, *origin);
            }

            origin.x += glyph_width;
        }
    }

    fn char_index(c: char) -> usize {
        Self::assert_printable(c);

        (c as usize) - ('!' as usize)
    }

    fn assert_printable(c: char) {
        assert!(
            Self::is_printable(c),
            "[GlyphMap] Pushing non-printable value (ASCII 0x{:x})",
            c as u32
        );
    }

    fn is_printable(c: char) -> bool {
        let repr = c as u32;
        repr >= 33 && repr <= 126
    }
}

pub struct Font<'a> {
    font: HalFont<'a>,
    pub glyph_size: PointF32,
    map: GlyphMap,
}

impl Font<'_> {
    pub fn new<'a>(ttf: &'a TtfContext, font_path: &CStr, size: f32) -> Font<'a> {
        let font = HalFont::new(ttf, font_path, size).expect("Cannot open font");
        assert!(
            font.is_mono(),
            "Font \"{}\" isn't fixed width",
            font.family()
        );

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

    pub fn free(&mut self, text: &str) {
        self.map.pop_str(text);
    }

    /// Calls [`GlyphMap::gc()`] on the contained map; see its
    /// documentation for more information.
    pub fn gc(&mut self, atlas: &mut Atlas) {
        self.map.gc(atlas);
    }

    pub fn draw(&self, text: &str, data: &GameData, origin: &mut PointF32) {
        self.map.draw(text, data, origin, self.glyph_size.x);
    }
}
