use halcyon::{
    color::Rgba,
    rect::{Point, PointF32, PointI32, RectF32},
    renderer::RendererRef,
    surface::Surface,
    texture::{Texture, TextureRef},
    traits::BlendMode,
};

use rectpack2d_rs::{
    best_bin_finder::CallbackResult,
    empty_space_allocators::DefaultEmptySpaces,
    empty_spaces::EmptySpaces,
    finders_interface::{Input, find_best_packing},
    rect_structs::RectXYWH,
};

use sdl3_sys::{blendmode::*, pixels::SDL_PIXELFORMAT_RGBA32, render::SDL_TEXTUREACCESS_TARGET};

fn to_frect(src: RectXYWH) -> RectF32 {
    RectF32::xywh(src.x as f32, src.y as f32, src.w as f32, src.h as f32)
}

fn to_r2d(src: PointI32) -> RectXYWH {
    RectXYWH::from_wh(src.x, src.y)
}

struct StagedEntry {
    surface: Surface,
    area: RectXYWH,
}

struct ActiveEntry {
    current: RectF32,
    staged: RectXYWH,
}

enum Data {
    Unused,
    Staged(StagedEntry),
    Active(ActiveEntry),
}

impl Data {
    fn staged(s: Surface) -> Data {
        let sz = s.size();
        Data::Staged(StagedEntry {
            surface: s,
            area: RectXYWH::from_wh(sz.x, sz.y),
        })
    }

    fn is_valid(&self) -> bool {
        !matches!(self, Self::Unused)
    }

    fn invalidate(&mut self) {
        *self = Self::Unused;
    }
}

/// Resize in accordance with expected required atlas capacity.
#[derive(Clone, Copy, Debug)]
pub struct AtlasId(u8);

pub struct RectWrapper<'a>(&'a mut RectXYWH);

impl<'a> From<&'a RectWrapper<'_>> for &'a RectXYWH {
    fn from(value: &'a RectWrapper) -> Self {
        value.0
    }
}

impl<'a> From<&'a mut RectWrapper<'_>> for &'a mut RectXYWH {
    fn from(value: &'a mut RectWrapper) -> Self {
        value.0
    }
}

pub struct Atlas {
    /// Stores both rectangles and staged surfaces.
    data: Vec<Data>,

    /// Necessary for `find_best_packing`.
    empty_spaces: EmptySpaces<DefaultEmptySpaces>,

    /// If `false`, [`Atlas::pack()`] is a no-op.
    /// This enables the caller to call said function in a loop without
    /// caring about anything else, while the atlas internally ensures
    /// it only executes when there's something to be done.
    pack_queued: bool,

    /// The atlas texture itself.
    pub texture: Option<Texture>,
}

impl Atlas {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            empty_spaces: EmptySpaces::default(),
            pack_queued: false,
            texture: None,
        }
    }

    pub fn push(&mut self, s: Surface) -> AtlasId {
        self.pack_queued = true;

        let data = Data::staged(s);
        let mut i = 0;

        while i < self.data.len() {
            let d = &mut self.data[i];
            if !d.is_valid() {
                *d = data;
                return AtlasId(i as _);
            }

            i += 1;
        }

        self.data.push(data);
        AtlasId(i as _)
    }

    pub fn remove(&mut self, i: AtlasId) {
        let Some(data) = self.data.get_mut(i.0 as usize) else {
            eprintln!("[Atlas] Trying to remove non-existent glyph ({i:?})");
            return;
        };

        data.invalidate();
    }

    fn create_texture(&mut self, rnd: RendererRef, size: PointI32) {
        let new_tex =
            Texture::new(rnd, SDL_PIXELFORMAT_RGBA32, SDL_TEXTUREACCESS_TARGET, size).unwrap();

        // PERF: This means "just copy the textures without doing any calculations"
        // which is absolutely sufficient for our case.
        new_tex.set_blend_mode(SDL_BLENDMODE_NONE);

        let old_tgt = unsafe { rnd.xchg_target(new_tex.as_ref()) }.unwrap();
        let old_col = rnd.xchg_draw_color_f32(Rgba::TRANSPARENT);

        let _ = rnd.clear();

        for d in &mut self.data {
            match d {
                Data::Unused => continue,
                Data::Staged(s) => {
                    let new_area = to_frect(s.area);

                    // Newly staged, just draw to the new texture.
                    let tex = Texture::from_surface(rnd, s.surface.as_ref()).unwrap();
                    let _ = rnd.draw(tex.as_ref(), None, Some(&new_area));

                    *d = Data::Active(ActiveEntry {
                        current: new_area,
                        staged: s.area,
                    })
                }
                Data::Active(a) => {
                    let new_area = to_frect(a.staged);

                    // Old, draw from previous rect to new one.
                    let _ = rnd.draw(
                        self.texture.as_ref().unwrap().as_ref(),
                        Some(&a.current),
                        Some(&new_area),
                    );

                    a.current = new_area;
                }
            }
        }

        new_tex.set_blend_mode(SDL_BLENDMODE_ADD_PREMULTIPLIED);
        self.texture = Some(new_tex);

        rnd.set_draw_color_f32(old_col);
        let _ = unsafe { rnd.set_target_opt(old_tgt) };
    }

    pub fn pack(&mut self, rnd: RendererRef) {
        if !self.pack_queued {
            return;
        }

        self.pack_queued = false;

        let input = Input {
            max_bin_side: 1024,
            discard_step: -4,
            handle_successful_insertion: |_| CallbackResult::ContinuePacking,
            handle_unsuccessful_insertion: |_| CallbackResult::AbortPacking,
        };

        fn fm(data: &mut Data) -> Option<&mut RectXYWH> {
            match data {
                Data::Unused => None,
                Data::Staged(s) => Some(&mut s.area),
                Data::Active(a) => Some(&mut a.staged),
            }
        }

        let size = find_best_packing(
            &mut self.empty_spaces,
            self.data.iter_mut().filter_map(fm),
            &input,
        );

        self.create_texture(rnd, Point::new(size.w as _, size.h as _));
    }

    fn extract_area(&self, id: AtlasId) -> RectF32 {
        match &self.data[id.0 as usize] {
            Data::Active(a) => a.current,
            _ => panic!("[Atlas] Trying to get area of invalid ID {}", id.0),
        }
    }

    pub fn draw(&self, rnd: RendererRef, id: AtlasId, dst: PointF32) {
        if let Some(tex) = &self.texture {
            let area = self.extract_area(id);
            let _ = rnd.draw(
                tex.as_ref(),
                Some(&area),
                Some(&RectF32::new(dst, area.size)),
            );
        }
    }

    pub fn draw_to(&self, rnd: RendererRef, id: AtlasId, dst: RectF32) {
        if let Some(tex) = &self.texture {
            let area = self.extract_area(id);
            let _ = rnd.draw(tex.as_ref(), Some(&area), Some(&dst));
        }
    }

    pub fn replace(&mut self, id: AtlasId, rnd: RendererRef, surf: Surface) {
        let d = &mut self.data[id.0 as usize];
        match d {
            Data::Active(a) => {
                let sz = surf.size();
                if sz.as_f32() == a.current.size {
                    // SAFETY: If a valid entry exists, the texture must exist as well.
                    let tex = self.texture.as_ref().unwrap().as_ref();
                    return Self::replace_exact_known(tex, a, rnd, surf);
                }

                self.pack_queued = true;
                *d = Data::Staged(StagedEntry {
                    surface: surf,
                    area: to_r2d(sz),
                });
            }
            _ => panic!("[Atlas] Trying to replace invalid ID {}", id.0),
        }
    }

    pub fn replace_exact(&mut self, id: AtlasId, rnd: RendererRef, s: Surface) {
        match &self.data[id.0 as usize] {
            Data::Active(a) => {
                // SAFETY: If a valid entry exists, the texture must exist as well.
                let tex = self.texture.as_ref().unwrap().as_ref();
                Self::replace_exact_known(tex, a, rnd, s)
            }
            _ => panic!("[Atlas] Trying to replace-exact invalid ID {}", id.0),
        }
    }

    fn replace_exact_known(tex: TextureRef, a: &ActiveEntry, rnd: RendererRef, s: Surface) {
        let rep = Texture::from_surface(rnd, s.as_ref()).unwrap();
        let dst = a.current;

        let old_blend = rnd.xchg_blend_mode(SDL_BLENDMODE_NONE);
        let old_tgt = unsafe { rnd.xchg_target(tex) }.unwrap();
        let old_draw = rnd.xchg_draw_color_f32(Rgba::TRANSPARENT);

        let _ = rnd.fill_rect(dst);
        let _ = rnd.draw(rep.as_ref(), None, Some(&dst));

        rnd.set_draw_color_f32(old_draw);
        let _ = unsafe { rnd.set_target_opt(old_tgt) };
        rnd.set_blend_mode(old_blend);
    }

    pub fn debug_draw(&self, rnd: RendererRef, origin: PointF32) {
        fn offset(mut r: RectF32, o: PointF32) -> RectF32 {
            r.pos.x += o.x;
            r.pos.y += o.y;
            r
        }

        fn fm(d: &Data, origin: PointF32) -> Option<RectF32> {
            match d {
                Data::Active(a) => Some(offset(a.current, origin)),
                _ => None,
            }
        }

        let vec = self
            .data
            .iter()
            .filter_map(|d| fm(d, origin))
            .collect::<Vec<_>>();

        let old_col = rnd.xchg_draw_color_f32(Rgba::RED);
        let _ = rnd.draw_rects(&vec);
        rnd.set_draw_color_f32(old_col);
    }
}
