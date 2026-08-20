#![allow(dead_code)]
#![windows_subsystem = "windows"]

use std::mem::ManuallyDrop;

use halcyon::{Result, error::Error, msgbox::ButtonLayout, subsystem::Video, ttf};

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
    chk!(msgbox::show(
        Severity::Error,
        ButtonLayout::LeftToRight,
        c"Game launch failed",
        &msg
    ));
}

fn run() -> Result {
    let ctx = halcyon::Context::new();
    let _vid = ManuallyDrop::new(Video::new(&ctx)?);
    let _ttf = ttf::Context::new()?;

    Game::init()?;
    Game::get_mut().main_loop();
    Game::drop();

    Ok(())
}

fn main() {
    run().unwrap_or_else(fail);
}
