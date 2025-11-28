#![allow(dead_code)]

use game::Game;
use halcyon::{context::Context, subsystem::Video, ttf::TtfContext, util::c_ptr_to_str};

mod anim;
mod atlas;
mod command;
mod console;
mod debug;
mod field;
mod game;
mod glyph_map;
mod util;

fn main() {
    let ctx = unsafe { Context::new() };
    let vid = Video::new(&ctx).expect("Could not initialize Halcyon's video subsystem");

    // Game segfaults without this. Oh well!
    let _ttf = TtfContext::new().expect("Could not initialize TTF");

    let mut game = Game::new(&vid)
        .map_err(|p| unsafe { c_ptr_to_str(p.as_ptr()) })
        .expect("Game initialization failed");

    game.main_loop();
}

#[cfg(test)]
mod tests {
    use sdl3_sys::keycode::SDLK_BACKSPACE;

    use crate::field::{Field, FieldAction};

    #[test]
    fn field() {
        let mut f = Field::new();

        // The embedded Unicode scalar values represent non-breaking spaces.
        assert!(f.process_str("Hello"));
        assert!(!f.process_str(" \u{A0} \u{A0} "));

        f.cursor = 5;
        assert!(f.process_str(" World!"));
        assert_eq!(f.text, "Hello World! \u{A0} \u{A0} ");

        // Remove one non-breaking space.
        f.cursor = 14;
        assert!(matches!(
            f.process_key(SDLK_BACKSPACE),
            FieldAction::TextRemoved
        ));

        assert_eq!(f.cursor, 13);
        assert_eq!(f.text, "Hello World!  \u{A0} ");
    }
}
