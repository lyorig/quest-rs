use halcyon::{
    Result,
    rect::{PointF32, PointI32},
    resource::Resource,
    ttf::{self, Text},
};

use crate::{
    atlas::Atlas,
    font::{
        Font,
        provider::{GlyphProvider, RefcountGlyphMap},
    },
    game::resources::Resources,
    util::resource_loader::ResourceLoader,
};

pub struct FontId(u8);

impl FontId {
    pub const UBUNTU_MONO: Self = Self(0);

    const fn as_index(&self) -> usize {
        self.0 as _
    }
}

/// Contains all fonts used in the game.
pub struct FontStoreGeneric<'t, GP: GlyphProvider> {
    array: [Font<'t, GP>; 1],
}

impl<GP: GlyphProvider> FontStoreGeneric<'_, GP> {
    pub fn new<'t>(
        ttf: &'t ttf::Context,
        rl: ResourceLoader,
        atlas: &mut Atlas,
    ) -> Result<FontStoreGeneric<'t, GP>> {
        let ubuntu = Font::new(ttf, &rl.resolve("UbuntuMono.ttf"), 32.0, atlas)?;
        Ok(FontStoreGeneric { array: [ubuntu] })
    }

    pub fn alloc(&mut self, id: FontId, text: &str, atlas: &mut Atlas) {
        self.array[id.as_index()].alloc(text, atlas);
    }

    pub fn free(&mut self, id: FontId, text: &str) {
        self.array[id.as_index()].free(text);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Font<'_, GP>> {
        self.array.iter()
    }

    pub fn draw(
        &self,
        id: FontId,
        res: &Resources,
        text: &str,
        origin: &mut PointF32,
        glyph_size: PointF32,
    ) {
        self.array[id.as_index()].draw(text, res, origin, glyph_size)
    }

    pub fn glyph_size(&self, id: FontId) -> PointI32 {
        let f = self.array[id.as_index()].font.as_ref();
        let tx = Text::new(f, "X").unwrap();
        tx.size()
    }
}

type GlyphMap = RefcountGlyphMap;
pub type FontStore<'a> = FontStoreGeneric<'a, GlyphMap>;
