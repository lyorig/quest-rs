use game::Game;
use halcyon::{context::Context, subsystem::Video};

mod atlas;
mod console;
mod debug;
mod game;

fn main() {
    let ctx = unsafe { Context::new() };
    let vid = Video::new(&ctx).expect("Could not initialize Halcyon's video subsystem");

    Game::new(&vid).main_loop();
}
