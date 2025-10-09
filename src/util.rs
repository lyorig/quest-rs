use std::{ffi::CStr, ptr::NonNull};

use halcyon::{
    defs::SdlResult,
    ttf::{Font, Text, TtfContext},
};

pub fn find_sized_font(ttf: &TtfContext, rel_path: &CStr, desired_height: f32) -> SdlResult<Font> {
    const INCR: f32 = 1.;
    let mut curr = 4.;

    while curr < 256. {
        let f = Font::new(ttf, rel_path, curr)?;

        curr += INCR;

        if (Text::new(&f, "X")?.size().y as f32) < desired_height {
            return Ok(f);
        }
    }

    Err(unsafe { NonNull::new_unchecked("Couldn't find suitable font".as_ptr().cast_mut().cast()) })
}
