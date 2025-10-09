#![allow(dead_code)]

use game::Game;
use halcyon::{context::Context, subsystem::Video, util::c_ptr_to_str};

mod atlas;
mod console;
mod debug;
mod field;
mod game;
mod resource_loader;
mod util;

fn main() {
    let ctx = unsafe { Context::new() };
    let vid = Video::new(&ctx).expect("Could not initialize Halcyon's video subsystem");

    let mut game = Game::new(&vid)
        .map_err(|p| unsafe { c_ptr_to_str(p.as_ptr()) })
        .expect("Game initialization failed");

    game.main_loop();
}
