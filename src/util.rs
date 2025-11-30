use std::{
    ffi::{CStr, c_char},
    ptr::NonNull,
};

use halcyon::{
    defs::SdlResult,
    ttf::{Font, Text, TtfContext},
};

/// Returns a `NonNull<c_char>`, for use with `SdlResult`s.
pub fn error_str(msg: &'static CStr) -> NonNull<c_char> {
    unsafe { NonNull::new_unchecked(msg.as_ptr().cast_mut()) }
}

pub fn find_sized_font<'a>(
    ttf: &'a TtfContext,
    rel_path: &CStr,
    desired_height: f32,
) -> SdlResult<Font<'a>> {
    const INCR: f32 = 1.0;
    let mut curr = 4.0;

    while curr < 256. {
        let f = Font::new(ttf, rel_path, curr)?;

        curr += INCR;

        if (Text::new(&f, "X")?.size().y as f32) >= desired_height {
            return Ok(f);
        }
    }

    Err(error_str(c"Couldn't find suitable font"))
}
