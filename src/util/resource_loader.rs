use std::ffi::{CStr, CString};

use halcyon::{defs::SdlResult, sdl_string::SdlString};

/// An abstraction over [`crate::base_path`] that provides
/// utility methods for constructing `Path`s relative to the
/// directory of the program that's running.
///
/// This struct is made up of a single `&'static Path`, so it's
/// safe to clone/copy.
pub struct ResourceLoader {
    root: SdlString,
}

impl ResourceLoader {
    pub fn from_pref() -> SdlResult<Self> {
        let root = halcyon::pref_path(c"cz.lyorig", c"quest")?;
        Ok(Self { root })
    }

    pub fn resolve(&self, path: &str) -> Box<CStr> {
        const NUL_ERROR: &str =
            "ResourceLoader::resolve() should never be given a Path with embedded NUL bytes";

        let total_len = self.root.len() + path.len();
        let mut vec = Vec::<u8>::with_capacity(total_len);
        vec.extend_from_slice(self.root.as_bytes());
        vec.extend_from_slice(path.as_bytes());

        let cs = unsafe { CString::from_vec_unchecked(vec) };
        cs.into_boxed_c_str()
    }
}
