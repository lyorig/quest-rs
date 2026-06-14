#![allow(dead_code)]
#![windows_subsystem = "windows"]

use halcyon::{defs::SdlResult, error::Error, ttf};

use crate::game::Game;

mod atlas;
mod console;
mod debug;
mod font;
mod game;
mod scene;
mod util;

fn fail(e: Error) {
    use halcyon::msgbox;
    use halcyon::msgbox::Severity;

    dprintln!("Initialization error: \"{e}\"");

    let msg = e.into_cstring();
    chk!(msgbox::show(
        Severity::Error,
        c"Game initialization failed",
        &msg,
    ));
}

fn run() -> SdlResult {
    let ttf = ttf::Context::new()?;
    let mut game = Game::new(&ttf)?;

    game.main_loop();

    Ok(())
}

fn main() {
    debug::init();

    if let Err(e) = run() {
        fail(e)
    }
}
