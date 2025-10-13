use halcyon::{
    color::Rgba,
    guard::{BlendModeGuard, DrawColorGuard, RenderTargetGuard},
    rect::{Point, PointF32, PointI32, RectF32},
    renderer::RendererRef,
    surface::Surface,
    texture::Texture,
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

struct Data {
    source: Option<Surface>,
    area: RectF32,
    staged: RectXYWH,
}

/// Resize in accordance with expected required atlas capacity.
#[derive(Clone, Copy)]
pub struct AtlasId(u8);

impl Data {
    pub fn new(s: Surface) -> Self {
        let sz = s.size();
        Self {
            source: Some(s),
            area: Default::default(),
            staged: RectXYWH::from_wh(sz.x, sz.y),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.staged.x != -1
    }

    pub fn invalidate(&mut self) {
        self.source = None;
        self.staged.x = -1;
    }
}

impl<'a> From<&'a Data> for &'a RectXYWH {
    fn from(value: &'a Data) -> Self {
        &value.staged
    }
}

impl<'a> From<&'a mut Data> for &'a mut RectXYWH {
    fn from(value: &'a mut Data) -> Self {
        &mut value.staged
    }
}

pub struct Atlas {
    /// Stores both rectangles and staged surfaces.
    data: Vec<Data>,

    /// Necessary for `find_best_packing`.
    empty_spaces: EmptySpaces<DefaultEmptySpaces>,

    /// If `false`, `Atlas::pack()` is a no-op.
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

        let data = Data::new(s);
        let mut i = 0;

        while i < self.data.len() {
            let foo = &mut self.data[i];
            if !foo.is_valid() {
                *foo = data;
                return AtlasId(i as _);
            }

            i += 1;
        }

        self.data.push(data);

        AtlasId(i as _)
    }

    pub fn remove(&mut self, i: AtlasId) {
        self.data[i.0 as usize].invalidate();
    }

    pub fn pack(&mut self, rnd: impl Into<RendererRef>) {
        let rnd: RendererRef = rnd.into();

        if !self.pack_queued {
            return;
        }

        self.pack_queued = false;

        let input = Input {
            max_bin_side: 4096,
            discard_step: -4,
            handle_successful_insertion: |_| CallbackResult::ContinuePacking,
            handle_unsuccessful_insertion: |_| CallbackResult::AbortPacking,
        };

        let size = find_best_packing(&mut self.empty_spaces, &mut self.data, &input);
        let mut new_tex = Texture::new(
            rnd,
            SDL_PIXELFORMAT_RGBA32,
            SDL_TEXTUREACCESS_TARGET,
            Point::new(size.w, size.h),
        )
        .unwrap();
        new_tex.set_blend_mode(SDL_BLENDMODE_ADD_PREMULTIPLIED);

        let _tgt = RenderTargetGuard::new(rnd, &new_tex);
        let _col = DrawColorGuard::new(rnd, Rgba::TRANSPARENT);

        let _ = rnd.clear();

        for d in &mut self.data {
            if !d.is_valid() {
                continue;
            }

            let new_area = to_frect(d.staged);

            match &d.source {
                Some(surf) => {
                    // Newly staged, just draw to the new texture.
                    let tex = Texture::from_surface(rnd, surf).unwrap();
                    let _ = rnd.draw(&tex, None, Some(&new_area));

                    d.source = None;
                }
                None => {
                    // Old, draw from previous rect to new one.
                    let _ = rnd.draw(
                        // SAFETY: If a Surface has been consumed, it's guaranteed to be residing on a Texture.
                        unsafe { self.texture.as_ref().unwrap_unchecked() },
                        Some(&d.area),
                        Some(&new_area),
                    );
                }
            }

            d.area = new_area;
        }

        self.texture = Some(new_tex);
    }

    pub fn draw(&self, rnd: impl Into<RendererRef>, id: AtlasId, dst: PointF32) {
        if let Some(tex) = &self.texture {
            let rnd: RendererRef = rnd.into();
            let area = self.data[id.0 as usize].area;

            let _ = rnd.draw(tex, Some(&area), Some(&RectF32::new(dst, area.size)));
        }
    }

    pub fn replace(&mut self, id: AtlasId, rnd: impl Into<RendererRef>, surf: Surface) {
        let d = &mut self.data[id.0 as usize];

        if Into::<PointF32>::into(surf.size()) == d.area.size {
            return self.replace_exact(id, rnd, surf);
        }

        self.pack_queued = true;
        d.staged = to_r2d(surf.size());
        d.source = Some(surf);
    }

    pub fn replace_exact(&mut self, id: AtlasId, rnd: impl Into<RendererRef>, s: Surface) {
        if let Some(tex) = &self.texture {
            let rnd: RendererRef = rnd.into();
            let rep = Texture::from_surface(rnd, &s).unwrap();
            let dst = self.data[id.0 as usize].area;

            let _blend = BlendModeGuard::new(rnd, SDL_BLENDMODE_NONE);
            let _col = DrawColorGuard::new(rnd, Rgba::TRANSPARENT);
            let _tgt = RenderTargetGuard::new(rnd, tex);

            let _ = rnd.fill_rect(dst);
            let _ = rnd.draw(&rep, None, Some(&dst));
        }
    }
}
