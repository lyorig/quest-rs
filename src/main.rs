#![allow(dead_code)]

use std::{ffi::c_char, process::ExitCode, ptr::NonNull};

use game::Game;
use halcyon::{context::Context, subsystem::Video, ttf::TtfContext, util::c_ptr_to_str};

mod atlas;
mod console;
mod debug;
mod font;
mod game;
mod util;

fn fail(msg: &str, err: NonNull<c_char>) -> ExitCode {
    let err = unsafe { c_ptr_to_str(err.as_ptr()) };
    dprint!("{msg}: \"{err}\"");
    ExitCode::FAILURE
}

macro_rules! ok_or_fail {
    ($call:expr, $fail:literal) => {
        match $call {
            Ok(val) => val,
            Err(e) => {
                return fail($fail, e);
            }
        }
    };
}

fn main() -> ExitCode {
    debug::init_epoch();

    let ctx = unsafe { Context::new() };

    let vid = ok_or_fail!(Video::new(&ctx), "Cannot initialize video subsystem");
    let _ttf = ok_or_fail!(
        TtfContext::new(),
        "Cannot initialize font manipulation facilities"
    );

    let mut game = ok_or_fail!(Game::new(&vid), "Cannot initialize game");

    game.main_loop();

    ExitCode::SUCCESS
}
