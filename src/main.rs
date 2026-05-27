#![allow(dead_code)]

use game::Game;
use halcyon::{context::Context, subsystem::Video, util::c_ptr_to_str};

mod atlas;
mod console;
mod debug;
mod font;
mod game;
mod util;

fn main() {
    let ctx = unsafe { Context::new() };
    let vid = Video::new(&ctx).expect("Video subsystem should be available");

    let mut game = Game::new(&vid)
        .map_err(|p| unsafe { c_ptr_to_str(p.as_ptr()) })
        .expect("Game initialization shouldn't fail");

    game.main_loop();
}
