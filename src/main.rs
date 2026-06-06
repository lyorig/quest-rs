#![allow(dead_code)]

use std::{process::ExitCode, ptr::NonNull};

use game::Game;
use halcyon::{context::Context, subsystem::Video, ttf::TtfContext};
use sdl3_sys::messagebox::{SDL_MESSAGEBOX_ERROR, SDL_ShowSimpleMessageBox};

mod atlas;
mod console;
mod debug;
mod font;
mod game;
mod util;

fn fail() {
    let msg = halcyon::error::get_owned();
    unsafe {
        SDL_ShowSimpleMessageBox(
            SDL_MESSAGEBOX_ERROR,
            c"Game init failed".as_ptr(),
            msg.as_ptr(),
            std::ptr::null_mut(),
        )
    };
}

fn do_init() -> Result<(), NonNull<i8>> {
    let ctx = unsafe { Context::new() };
    let vid = Video::new(&ctx)?;
    let _ttf = TtfContext::new()?;

    let mut game = Game::new(&vid)?;
    game.main_loop();

    Ok(())
}

fn main() -> ExitCode {
    debug::init_epoch();

    match do_init() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            fail();
            ExitCode::FAILURE
        }
    }
}
