use halcyon::{rect::PointF32, resource_loader::ResourceLoader};

use crate::{atlas::Atlas, font::Font, game::resources::GameResources};

pub struct FontId(usize);

impl FontId {
    pub const UBUNTU_MONO: Self = Self(0);
}

/// Contains all fonts used in the game.
pub struct FontStore {
    array: [Font; 1],
}

impl FontStore {
    /// SAFETY: Make sure a [`halcyon::ttf::TtfContext`] is active.
    pub unsafe fn new(rl: ResourceLoader) -> FontStore {
        let ubuntu = unsafe { Font::new(&rl.resolve("../../bin/assets/UbuntuMono.ttf"), 32.0) };
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
}
