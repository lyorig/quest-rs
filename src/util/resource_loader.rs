use std::ffi::{CStr, CString};

use halcyon::{Result, boxed};

/// A simple string wrapper that provides utility methods for
/// constructing a [`std::path::Path`] relative to the directory
/// of the program that's running.
pub struct ResourceLoader {
    root: boxed::Box<str>,
}

impl ResourceLoader {
    pub fn from_pref() -> Result<Self> {
        halcyon::fs::pref_path(c"cz.lyorig", c"quest").map(|s| Self {
            root: s.into_boxed_str(),
        })
    }

    pub fn resolve(&self, path: &str) -> Box<CStr> {
        let total_len = self.root.len() + path.len() + 1;

        let mut vec = Vec::<u8>::with_capacity(total_len);
        vec.extend_from_slice(self.root.as_bytes());
        vec.extend_from_slice(path.as_bytes());
        vec.push(b'\0');

        let cs = unsafe { CString::from_vec_with_nul_unchecked(vec) };
        cs.into_boxed_c_str()
    }
}
