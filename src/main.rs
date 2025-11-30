#![allow(dead_code)]

use game::Game;
use halcyon::{context::Context, subsystem::Video, ttf::TtfContext, util::c_ptr_to_str};

mod anim;
mod atlas;
mod console;
mod debug;
mod game;
mod glyph_map;
mod util;

fn main() {
    let ctx = unsafe { Context::new() };
    let vid = Video::new(&ctx).expect("Could not initialize Halcyon's video subsystem");
    let ttf = TtfContext::new().expect("Should be able to initialize TTF");

    let mut game = unsafe { Game::new(&vid, &ttf) }
        .map_err(|p| unsafe { c_ptr_to_str(p.as_ptr()) })
        .expect("Game initialization failed");

    game.main_loop();
}
