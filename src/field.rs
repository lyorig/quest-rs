use halcyon::clipboard;
use sdl3_sys::keycode::*;

pub struct Field {
    pub text: String,
    pub cursor: usize,
}

pub enum FieldAction {
    Noop,
    TextAdded,
    TextRemoved,
    CursorMoved,
}

impl Field {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
        }
    }

    pub fn process_str(&mut self, inp: &str) -> bool {
        self.text.insert_str(self.cursor_byte_index(), inp);
        self.cursor += inp.chars().count();

        println!("Cursor now at pos {}", self.cursor);

        inp.chars().any(|c| !c.is_whitespace())
    }

    pub fn process_key(&mut self, k: SDL_Keycode) -> FieldAction {
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

                        if curr.is_whitespace() {
                            while end != self.text.len() && self.char_at(end).is_whitespace() {
                                end += 1;
                            }
                        } else if curr.is_alphabetic() {
                            while end != self.text.len() && self.char_at(end).is_alphabetic() {
                                end += 1;
                            }
                        } else {
                            let c = self.char_at(end);
                            while end != self.text.len() && !c.is_alphabetic() && c != ' ' {
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
                            let c = self.char_at(begin);
                            while begin != 0 && !c.is_alphabetic() && c != ' ' {
                                begin -= 1;
                            }

                            if begin != 0 {
                                begin += 1;
                            }
                        }

                        self.cursor -= end - begin;
                    }

                    self.text
                        .replace_range(self.byte_index(begin)..self.byte_index(end), "");

                    return FieldAction::TextRemoved;
                } else {
                    if self.cursor != 0 {
                        self.cursor -= 1;
                    }

                    self.text.remove(self.cursor_byte_index());

                    return FieldAction::TextRemoved;
                }
            }

            SDLK_LEFT => {
                if self.cursor != 0 {
                    self.cursor -= 1;
                }

                return FieldAction::CursorMoved;
            }

            SDLK_RIGHT => {
                self.cursor = (self.cursor + 1).min(self.text.len());
                return FieldAction::CursorMoved;
            }

            SDLK_TAB => {
                self.text.insert_str(
                    self.cursor,
                    &std::iter::repeat_n(' ', 4).collect::<String>(),
                );
                self.cursor += 4;

                return FieldAction::TextAdded;
            }

            SDLK_V => {
                if halcyon::keyboard::mod_state() & SDL_KMOD_CTRL != 0 && clipboard::has_text() {
                    let clip = clipboard::text();
                    let size = clip.len();

                    self.text.insert_str(self.cursor, &clip);

                    self.cursor += size;

                    return FieldAction::TextAdded;
                }
            }
            _ => (),
        };

        FieldAction::Noop
    }

    pub fn trim(&mut self, off: usize) {
        if self.cursor > off {
            self.cursor -= self.cursor - off;
        }

        self.text.remove(off);
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    fn char_at(&self, i: usize) -> char {
        self.text.chars().nth(i).unwrap()
    }

    fn byte_index(&self, i: usize) -> usize {
        self.text
            .char_indices()
            .nth(i)
            .map(|f| f.0)
            .unwrap_or(self.text.len())
    }

    fn cursor_byte_index(&self) -> usize {
        self.byte_index(self.cursor)
    }
}
