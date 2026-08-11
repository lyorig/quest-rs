use std::ffi::{CStr, CString};

use halcyon::{Result, string::String};

/// A simple string wrapper that provides utility methods for
/// constructing a [`std::path::Path`] relative to the directory
/// of the program that's running.
pub struct ResourceLoader {
    root: String,
}

impl ResourceLoader {
    pub fn from_pref() -> Result<Self> {
        let root = halcyon::pref_path(c"cz.lyorig", c"quest")?;
        Ok(Self { root })
    }

    pub fn resolve(&self, path: &str) -> Box<CStr> {
        const NUL_ERROR: &str =
            "ResourceLoader::resolve() should never be given a Path with embedded NUL bytes";

        let root = self.root.to_str();
        let total_len = root.len() + path.len() + 1;

        let mut vec = Vec::<u8>::with_capacity(total_len);
        vec.extend_from_slice(root.as_bytes());
        vec.extend_from_slice(path.as_bytes());
        vec.push(b'\0');

        let cs = unsafe { CString::from_vec_with_nul_unchecked(vec) };
        cs.into_boxed_c_str()
    }
}
