use std::mem::MaybeUninit;

use halcyon::{renderer::RendererRef, surface::Surface, texture::Texture};
use rectpack2d_rs::{
    best_bin_finder::CallbackResult,
    empty_space_allocators::DefaultEmptySpaces,
    empty_spaces::EmptySpaces,
    finders_interface::{Input, find_best_packing},
    rect_structs::RectXYWH,
};
use sdl3_sys::{pixels::SDL_PIXELFORMAT_RGBA32, rect::SDL_FRect, render::SDL_TEXTUREACCESS_TARGET};

struct Data {
    source: Option<Surface>,
    area: SDL_FRect,
    staged: RectXYWH,
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

    pub fn pack(&mut self, rnd: RendererRef) {
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
            (size.w, size.h),
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
}

fn to_frect(src: RectXYWH) -> SDL_FRect {
    SDL_FRect {
        x: src.x as f32,
        y: src.y as f32,
        w: src.w as f32,
        h: src.h as f32,
    }
}
