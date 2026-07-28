#![allow(dead_code)]
#![windows_subsystem = "windows"]

use halcyon::{Result, error::Error, subsystem::Video, ttf};

use crate::game::Game;

mod atlas;
mod console;
mod debug;
mod font;
mod game;
mod scene;
mod ui;
mod util;

fn fail(e: Error) {
    use halcyon::msgbox;
    use halcyon::msgbox::Severity;

    dprintln!("Launch error: \"{e}\"");

    let msg = e.into_cstring();
    chk!(msgbox::show(Severity::Error, c"Game launch failed", &msg));
}

fn run() -> Result {
    let ctx = halcyon::Context::new();
    let _vid = Video::new(&ctx)?;

    let ttf = ttf::Context::new()?;
    let mut game = Game::new(&ttf)?;

    game.main_loop();

    Ok(())
}

fn main() {
    debug::init();

    run().unwrap_or_else(fail);
}
