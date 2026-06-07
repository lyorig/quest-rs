#![allow(dead_code)]
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use game::Game;
use halcyon::{context::Context, error::Error, subsystem::Video, ttf::TtfContext};
use sdl3_sys::messagebox::SDL_MESSAGEBOX_ERROR;

mod atlas;
mod console;
mod debug;
mod font;
mod game;
mod util;

fn fail(e: Error) {
    dprintln!("Initialization error: \"{e}\"");

    util::chk(halcyon::msgbox::show(
        SDL_MESSAGEBOX_ERROR,
        c"Game initialization failed",
        e.as_cstr(),
    ));
}

fn do_init() -> Result<(), Error> {
    let ctx = unsafe { Context::new() };
    let vid = Video::new(&ctx)?;
    let _ttf = TtfContext::new()?;

    let mut game = Game::new(&vid)?;
    game.main_loop();

    Ok(())
}

fn main() -> ExitCode {
    debug::init();

    match do_init() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            fail(e);
            ExitCode::FAILURE
        }
    }
}
