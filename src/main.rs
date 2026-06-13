#![allow(dead_code)]
#![windows_subsystem = "windows"]

use std::sync::Mutex;

use game::Game;
use halcyon::{
    context::Context, defs::SdlResult, error::Error, event::Event, subsystem::Video,
    ttf::TtfContext,
};
use sdl3_main::{AppResult, AppResultWithState};
use sdl3_sys::events::SDL_Event;

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

fn do_init() -> SdlResult<MyAppState> {
    let _ctx = unsafe { Context::new() };
    let _vid = Video::new(&_ctx)?;
    let _ttf = TtfContext::new()?;

    let game = Game::new()?;

    let ret = MyAppState {
        game,
        _ctx,
        _vid,
        _ttf,
    };

    Ok(ret)
}

// NOTE: The drop order is important here!
struct MyAppState {
    game: Game,
    _ttf: TtfContext,
    _vid: Video,
    _ctx: Context,
}

unsafe impl Send for MyAppState {}

#[sdl3_main::app_impl]
impl MyAppState {
    fn app_init() -> AppResultWithState<Box<Mutex<Self>>> {
        debug::init();

        match do_init() {
            Ok(mas) => {
                let ret = Box::new(Mutex::new(mas));
                AppResultWithState::Continue(ret)
            }
            Err(e) => {
                fail(e);
                AppResultWithState::Failure(None)
            }
        }
    }

    fn app_iterate(&mut self) -> AppResult {
        self.game.iterate()
    }

    fn app_event(&mut self, event: SDL_Event) -> AppResult {
        let evt = Event::from(event);
        self.game.process_event(evt)
    }

    fn app_quit(&mut self) {}
}
