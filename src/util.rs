// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `oic` utilities
use error::{Error, Result};
use std::convert::{TryFrom, TryInto};
use std::os::raw::c_char;
use std::ptr;
use std::slice;

/// Holds a pointer and a length for ODPI-C strings.
#[derive(Clone, Copy, Debug)]
pub struct ODPIStr {
    /// A pointer to the head of the FFI string.
    ptr: *const c_char,
    /// The length of the string.
    len: u32,
}

impl ODPIStr {
    /// Create a new `ODPIStr`.
    pub fn new(ptr: *const c_char, len: u32) -> Self {
        Self { ptr: ptr, len: len }
    }

    /// Get the `ptr` value.
    pub fn ptr(&self) -> *const c_char {
        self.ptr
    }

    /// Get the `len` value.
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Is the string empty.
    pub fn is_empty(&self) -> bool {
        self.ptr.is_null() || self.len == 0
    }
}

impl Default for ODPIStr {
    fn default() -> Self {
        Self {
            ptr: ptr::null(),
            len: 0,
        }
    }
}

impl<'a> TryFrom<Option<&'a str>> for ODPIStr {
    type Error = Error;

    fn try_from(opt_s: Option<&str>) -> Result<Self> {
        match opt_s {
            Some(s) => TryFrom::try_from(s),
            None => Ok(Default::default()),
        }
    }
}

impl<'a> TryFrom<&'a str> for ODPIStr {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        let s_len: u32 = TryInto::try_into(s.len())?;
        Ok(Self {
            ptr: s.as_ptr() as *const c_char,
            len: s_len,
        })
    }
}

impl TryFrom<String> for ODPIStr {
    type Error = Error;

    fn try_from(s: String) -> Result<Self> {
        let s_len: u32 = TryInto::try_into(s.len())?;
        Ok(Self {
            ptr: s.as_ptr() as *const c_char,
            len: s_len,
        })
    }
}

impl From<ODPIStr> for String {
    fn from(s: ODPIStr) -> Self {
        if s.ptr.is_null() {
            "".to_string()
        } else {
            let vec = unsafe { slice::from_raw_parts(s.ptr as *mut u8, s.len as usize) };
            Self::from_utf8_lossy(vec).into_owned()
        }
    }
}
