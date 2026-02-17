use halcyon::{rect::PointF32, resource_loader::ResourceLoader, ttf::TtfContext};

use crate::{atlas::Atlas, font::Font, game::resources::GameResources};

pub struct FontId(usize);

impl FontId {
    pub const UBUNTU_MONO: Self = Self(0);
}

pub struct FontStore<'a> {
    array: [Font<'a>; 1],
}

impl FontStore<'_> {
    pub fn new<'a>(ttf: &'a TtfContext, rl: ResourceLoader) -> FontStore<'a> {
        let ubuntu = Font::new(ttf, &rl.resolve("../../bin/assets/UbuntuMono.ttf"), 32.0);

        FontStore { array: [ubuntu] }
    }

    pub fn alloc(&mut self, id: FontId, text: &str, atlas: &mut Atlas) {
        self.array[id.0].alloc(text, atlas);
    }

    pub fn free(&mut self, id: FontId, text: &str) {
        self.array[id.0].free(text);
    }

    pub fn gc(&mut self, id: FontId, atlas: &mut Atlas) {
        self.array[id.0].gc(atlas);
    }

    pub fn gc_all(&mut self, atlas: &mut Atlas) {
        self.array.iter_mut().for_each(|f| f.gc(atlas))
    }

    pub fn draw(
        &self,
        id: FontId,
        res: &GameResources,
        text: &str,
        origin: &mut PointF32,
        glyph_size: PointF32,
    ) {
        self.array[id.0].draw(text, res, origin, glyph_size)
    }

    pub fn get(&self, id: FontId) -> &Font<'_> {
        &self.array[id.0]
    }
}
