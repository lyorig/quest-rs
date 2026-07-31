use halcyon::clipboard;
use sdl3_sys::keycode::*;

use crate::{font::store::FontId, game::resources::Resources};

const MAX_CHARS: usize = 32;

/// Represents [`super::Console`]'s text input field.
///
/// This is an intermediary between the console's graphical
/// presentation of what's being typed, and the in-memory one.
/// As such, many functions are written in a way that perform
/// the necessary internal logic, but also return values that
/// act as hints that the console uses to find out whether it's
/// necessary to act on whatever processing just took place
/// (mainly concerning re-constructing and drawing textures).
///
/// There may be many edge cases left in here; for now, I'm happy
/// with a state of "oh well, at least it works", but a future
/// analysis of an optimal way to perform UTF-8 operations may
/// very well be in order.
pub struct Field {
    /// The current contents of the input.
    pub text: String,

    /// The **character** index of where the cursor lies.
    /// Special care must be taken when using this variable
    /// within Rust's [`String`] API, since they expect indices
    /// to lie on a **byte** boundary. Use [`Field::cursor_byte_index`]
    /// in order to find out which byte index it corresponds to.
    pub cursor: usize,
}

impl Field {
    pub const fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
        }
    }

    pub fn process_str(&mut self, inp: &str, gd: &mut Resources) {
        let fil: String = inp.chars().filter(char::is_ascii).collect();

        self.text.insert_str(self.cursor_byte_index(), &fil);
        self.cursor += fil.len();

        gd.font_alloc(FontId::UBUNTU_MONO, &fil);
    }

    /// Returns whether the cursor should be moved.
    pub fn process_key(&mut self, k: SDL_Keycode, gd: &mut Resources) -> bool {
        match k {
            SDLK_BACKSPACE => 'a: {
                if self.text.is_empty() {
                    break 'a;
                }

                if halcyon::keyboard::mod_state() & SDL_KMOD_CTRL != 0 {
                    let mut begin: usize;
                    let mut end: usize;

                    if self.cursor == 0 {
                        (begin, end) = (0, 0);

                        let curr = self.char_at(begin);
                        let chars = self.text.len();

                        if curr.is_whitespace() {
                            while end != chars && self.char_at(end).is_whitespace() {
                                end += 1;
                            }
                        } else if curr.is_alphabetic() {
                            while end != chars && self.char_at(end).is_alphabetic() {
                                end += 1;
                            }
                        } else {
                            while end != chars {
                                let c = self.char_at(end);
                                if c.is_alphabetic() || c == ' ' {
                                    break;
                                }

                                end += 1;
                            }
                        }
                    } else {
                        begin = self.cursor - 1;
                        end = self.cursor;

                        let curr = self.char_at(begin);

                        if curr.is_whitespace() {
                            while begin != 0 && self.char_at(begin).is_whitespace() {
                                begin -= 1;
                            }
                        } else if curr.is_alphabetic() {
                            while begin != 0 && self.char_at(begin).is_alphabetic() {
                                begin -= 1;
                            }
                        } else {
                            let mut c = self.char_at(begin);
                            while begin != 0 && !c.is_alphabetic() && c != ' ' {
                                begin -= 1;
                                c = self.char_at(begin);
                            }

                            if begin != 0 {
                                begin += 1;
                            }
                        }

                        self.cursor -= end - begin;
                    }

                    begin = self.byte_index(begin);
                    end = self.byte_index(end);

                    self.text
                        .replace_range(self.byte_index(begin)..self.byte_index(end), "");

                    return true;
                } else {
                    if self.cursor != 0 {
                        self.cursor -= 1;
                    }

                    self.text.remove(self.cursor_byte_index());

                    return true;
                }
            }

            SDLK_LEFT => {
                if self.cursor != 0 {
                    self.cursor -= 1;
                }

                return true;
            }

            SDLK_RIGHT => {
                self.cursor = (self.cursor + 1).min(self.text.len());
                return true;
            }

            SDLK_TAB => {
                self.text.insert_str(self.cursor, "    ");
                self.cursor += 4;

                return true;
            }

            SDLK_V => {
                if halcyon::keyboard::mod_state() & SDL_KMOD_CTRL != 0 && clipboard::has_text() {
                    let clip = clipboard::text();
                    self.process_str(clip.to_str(), gd);

                    return true;
                }
            }
            _ => return false,
        }

        false
    }

    pub fn trim_check(&mut self) {
        let c = self.text.len();
        if c > MAX_CHARS {
            self.text
                .replace_range(self.text.char_indices().nth(MAX_CHARS).unwrap().0.., "");
            self.cursor = MAX_CHARS;
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    const fn char_at(&self, i: usize) -> char {
        self.text.as_bytes()[i] as char
    }

    const fn byte_index(&self, i: usize) -> usize {
        // PERF: ASCII-only, so every char is 1 byte.
        i
    }

    const fn cursor_byte_index(&self) -> usize {
        self.byte_index(self.cursor)
    }
}
