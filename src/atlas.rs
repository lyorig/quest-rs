use std::mem::MaybeUninit;

use halcyon::{
    rect::{Point, PointF32, RectF32},
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

use sdl3_sys::{pixels::SDL_PIXELFORMAT_RGBA32, render::SDL_TEXTUREACCESS_TARGET};

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
    pack_queued: bool,

    /// The atlas texture itself.
    texture: MaybeUninit<Texture>,
}

impl Atlas {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            empty_spaces: EmptySpaces::default(),
            pack_queued: false,
            texture: MaybeUninit::uninit(),
        }
    }

    pub fn push(&mut self, s: Surface) -> AtlasId {
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
        if self.data.len() <= i.0 as _ {
            return;
        }

        self.data[i.0 as usize].invalidate();
    }

    pub fn pack(&mut self, rnd: impl Into<RendererRef>) {
        let rnd = rnd.into();

        if !self.pack_queued {
            return;
        }

        self.pack_queued = false;

        let input = Input {
            max_bin_side: 4096,
            discard_step: 4,
            handle_successful_insertion: |_| CallbackResult::ContinuePacking,
            handle_unsuccessful_insertion: |_| CallbackResult::AbortPacking,
        };

        let size = find_best_packing(&mut self.empty_spaces, &mut self.data, &input);
        let new_tex = Texture::new(
            rnd,
            SDL_PIXELFORMAT_RGBA32,
            SDL_TEXTUREACCESS_TARGET,
            Point::new(size.w, size.h),
        )
        .unwrap();

        rnd.set_target(*new_tex).expect("Cannot set render target");

        for d in &mut self.data {
            let new_area = to_frect(d.staged);

            match &d.source {
                Some(surf) => {
                    // Newly staged, just draw to the new texture.
                    let tex = Texture::from_surface(rnd, surf).unwrap();

                    rnd.draw(&tex, None, Some(&new_area))
                        .expect("Cannot draw new atlas texture");

                    d.source = None;
                    d.area = new_area;
                }
                None => {
                    // Old, draw from previous rect to new one.
                    rnd.draw(
                        unsafe { self.texture.assume_init_ref() },
                        None,
                        Some(&new_area),
                    )
                    .expect("Cannot draw old atlas texture");

                    d.area = new_area;
                }
            }
        }

        let _ = rnd.reset_target();

        self.texture.write(new_tex);
    }

    pub fn draw(&self, rnd: impl Into<RendererRef>, id: AtlasId, dst: PointF32) {
        let rnd: RendererRef = rnd.into();
        let area = self.data[id.0 as usize].area;

        let _ = rnd.draw(
            unsafe { self.texture.assume_init_ref() },
            Some(&area),
            Some(&RectF32 {
                pos: dst,
                size: area.size,
            }),
        );
    }

    pub fn draw_part(
        &self,
        rnd: impl Into<RendererRef>,
        id: AtlasId,
        mut src: RectF32,
        dst: PointF32,
    ) {
        let rnd: RendererRef = rnd.into();
        let area = self.data[id.0 as usize].area;

        src.pos.x += area.pos.x;
        src.pos.y += area.pos.y;

        let _ = rnd.draw(
            unsafe { self.texture.assume_init_ref() },
            Some(&src),
            Some(&RectF32 {
                pos: dst,
                size: area.size,
            }),
        );
    }

    pub fn area(&self, id: AtlasId) -> RectF32 {
        self.data[id.0 as usize].area
    }

    pub fn replace(&mut self, id: AtlasId, rnd: impl Into<RendererRef>, surf: Surface) {
        let d = &self.data[id.0 as usize];

        if Into::<PointF32>::into(surf.size()) == d.area.size {
            return self.replace_exact(id, rnd, surf);
        }
    }

    pub fn replace_exact(&mut self, id: AtlasId, rnd: impl Into<RendererRef>, s: Surface) {
        let rnd: RendererRef = rnd.into();

        let _ = rnd.set_target(unsafe { self.texture.assume_init_ref() });
        let tex = Texture::from_surface(rnd, &s).unwrap();

        let _ = rnd.draw(&tex, None, Some(&self.data[id.0 as usize].area));

        let _ = rnd.reset_target();
    }
}

fn to_frect(src: RectXYWH) -> RectF32 {
    RectF32::xywh(src.x as f32, src.y as f32, src.w as f32, src.h as f32)
}
