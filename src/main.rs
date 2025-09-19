use game::Game;
use halcyon::{context::Context, subsystem::Video};

mod atlas;
mod debugger;
mod game;

fn main() {
    let ctx = unsafe { Context::new() };
    let _vid = Video::new(&ctx);

    Game::new().main_loop();
}
