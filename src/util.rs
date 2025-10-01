use std::ffi::CStr;

use halcyon::ttf::{Font, Text, TtfContext};

pub fn find_sized_font(ttf: &TtfContext, rel_path: &CStr, desired_height: i32) -> Font {
    const INCR: f32 = 1.;
    let mut curr = 4.;

    loop {
        let f = Font::new(ttf, rel_path, curr).expect("Cannot open font");
        curr += INCR;

        if Text::new(&f, "X")
            .expect("Text construction failed??")
            .size()
            .y
            < desired_height
        {
            return f;
        }
    }
}
