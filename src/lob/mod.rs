// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! LOB handles are used to represent large objects (CLOB, BLOB, NCLOB, BFILE). Both persistent and
//! temporary large objects can be represented. LOB handles can be created by calling the function
//! `Connection::new_temp_lob()` or are created implicitly when a variable of type
//! `DPI_ORACLE_TYPE_CLOB`, `DPI_ORACLE_TYPE_NCLOB`, `DPI_ORACLE_TYPE_BLOB` or
//! `DPI_ORACLE_TYPE_BFILE` is created and are destroyed when the last reference is released by
//! calling the function `Lob::release()`. They are used for reading and writing data to the
//! database in smaller pieces than is contained in the large object.
use error::{ErrorKind, Result};
use odpi::externs;
use odpi::opaque::ODPILob;
use std::convert::TryFrom;
use std::ptr;
use util::ODPIStr;

/// LOB handles are used to represent large objects (CLOB, BLOB, NCLOB, BFILE).
pub struct Lob {
    /// The ODPI-C LOB pointer.
    inner: *mut ODPILob,
}

impl Lob {
    #[doc(hidden)]
    pub fn inner(&self) -> *mut ODPILob {
        self.inner
    }

    /// Closes the LOB resource. This should be done when a batch of writes has been completed so
    /// that the indexes associated with the LOB can be updated. It should only be performed if a
    /// call to function `Lob::open_resource()` has been performed.
    pub fn close_resource(&self) -> Result<()> {
        try_dpi!(
            externs::dpiLob_closeResource(self.inner),
            Ok(()),
            ErrorKind::Lob("dpiLob_closeResource".to_string())
        )
    }

    /// Creates an independent copy of a LOB and returns a reference to the newly created LOB. This
    /// reference should be released as soon as it is no longer needed.
    pub fn copy(&self, dst: &mut Self) -> Result<()> {
        try_dpi!(
            externs::dpiLob_copy(self.inner, &mut dst.inner),
            Ok(()),
            ErrorKind::Lob("dpiLob_copy".to_string())
        )
    }

    /// Flush or write all buffers for this LOB to the server.
    pub fn flush_buffer(&self) -> Result<()> {
        try_dpi!(
            externs::dpiLob_flushBuffer(self.inner),
            Ok(()),
            ErrorKind::Lob("dpiLob_flushBuffer".to_string())
        )
    }

    /// Returns the size of the buffer needed to hold the number of characters specified for a
    /// buffer of the type associated with the LOB. If the LOB does not refer to a character LOB the
    ///  value is returned unchanged.
    ///
    /// * `size_in_chars` - the number of characters for which a buffer size needs to be determined.
    pub fn get_buffer_size(&self, size_in_chars: u64) -> Result<u64> {
        let mut size_in_bytes = 0;
        try_dpi!(
            externs::dpiLob_getBufferSize(self.inner, size_in_chars, &mut size_in_bytes),
            Ok(size_in_bytes),
            ErrorKind::Lob("dpiLob_getBufferSize".to_string())
        )
    }

    /// Returns the chunk size of the internal LOB. Reading and writing to the LOB in multiples of
    /// this size will improve performance.
    pub fn get_chunk_size(&self) -> Result<u32> {
        let mut size = 0;
        try_dpi!(
            externs::dpiLob_getChunkSize(self.inner, &mut size),
            Ok(size),
            ErrorKind::Lob("dpiLob_getChunkSize".to_string())
        )
    }

    /// Returns the directory alias name and file name for a BFILE type LOB.
    ///
    /// Returns a `(String, String)` tuple representing the directory alias and filename.
    pub fn get_directory_and_filename(&self) -> Result<(String, String)> {
        let mut dir_alias = ptr::null();
        let mut dir_alias_len = 0;
        let mut filename = ptr::null();
        let mut filename_len = 0;

        try_dpi!(
            externs::dpiLob_getDirectoryAndFileName(
                self.inner,
                &mut dir_alias,
                &mut dir_alias_len,
                &mut filename,
                &mut filename_len
            ),
            {
                let da = if dir_alias.is_null() {
                    "".to_string()
                } else {
                    let dir_alias_s = ODPIStr::new(dir_alias, dir_alias_len);
                    dir_alias_s.into()
                };
                let fn_str = if filename.is_null() {
                    "".to_string()
                } else {
                    let filename_s = ODPIStr::new(filename, filename_len);
                    filename_s.into()
                };
                Ok((da, fn_str))
            },
            ErrorKind::Lob("dpiLog_getDirectoryAndFilename".to_string())
        )
    }

    /// Returns a bool value indicating if the file referenced by the BFILE type LOB exists.
    pub fn get_file_exists(&self) -> Result<bool> {
        let mut exists = 0;
        try_dpi!(
            externs::dpiLob_getFileExists(self.inner, &mut exists),
            Ok(exists == 1),
            ErrorKind::Lob("dpiLob_getFileExists".to_string())
        )
    }

    /// Returns a boolean value indicating if the LOB resource has been opened by making a call to
    /// the function Lob::open_resource() or not.
    pub fn get_is_resource_open(&self) -> Result<bool> {
        let mut open = 0;
        try_dpi!(
            externs::dpiLob_getIsResourceOpen(self.inner, &mut open),
            Ok(open == 1),
            ErrorKind::Lob("dpiLob_getIsResourceOpen".to_string())
        )
    }

    /// Returns the size of the data stored in the LOB. For character LOBs the size is in
    /// characters; for binary LOBs the size is in bytes.
    pub fn get_size(&self) -> Result<u64> {
        let mut size = 0;
        try_dpi!(
            externs::dpiLob_getSize(self.inner, &mut size),
            Ok(size),
            ErrorKind::Lob("dpiLob_getSize".to_string())
        )
    }

    /// Opens the LOB resource for writing. This will improve performance when writing to the LOB in
    /// chunks and there are functional or extensible indexes associated with the LOB. If this
    /// function is not called, the LOB resource will be opened and closed for each write that is
    /// performed. A call to the function `Lob::close_resource()` should be done before performing a
    /// call to the function `Connection::commit()`.
    pub fn open_resource(&self) -> Result<()> {
        try_dpi!(
            externs::dpiLob_openResource(self.inner),
            Ok(()),
            ErrorKind::Lob("dpiLob_openResource".to_string())
        )
    }

    /// Reads data from the LOB at the specified offset into the provided buffer.
    pub fn read_bytes(&self, offset: u64, length: u64) -> Result<Vec<i8>> {
        let length_usize: usize = TryFrom::try_from(length)?;
        let mut buffer: Vec<i8> = Vec::with_capacity(length_usize);
        let buf_ptr = buffer.as_mut_ptr();
        let mut buf_len = length;

        try_dpi!(
            externs::dpiLob_readBytes(self.inner, offset, length, buf_ptr, &mut buf_len),
            {
                unsafe { buffer.set_len(TryFrom::try_from(buf_len)?) };
                Ok(buffer)
            },
            ErrorKind::Lob("dpiLob_readBytes".to_string())
        )
    }

    /// Sets the directory alias name and file name for a BFILE type LOB.
    ///
    /// * `directory` - the name of the directory alias.
    /// * `filename` - the name of the file.
    pub fn set_directory_and_filename(&self, directory: &str, filename: &str) -> Result<()> {
        let dir_s: ODPIStr = TryFrom::try_from(directory)?;
        let fn_s: ODPIStr = TryFrom::try_from(filename)?;

        try_dpi!(
            externs::dpiLob_setDirectoryAndFileName(
                self.inner,
                dir_s.ptr(),
                dir_s.len(),
                fn_s.ptr(),
                fn_s.len()
            ),
            Ok(()),
            ErrorKind::Lob("dpiLob_setDirectoryAndFileName".to_string())
        )
    }

    /// Replaces all of the data in the LOB with the contents of the provided buffer. The LOB will
    /// first be cleared and then the provided data will be written.
    ///
    /// * `buffer` - the buffer from which the data is written.
    pub fn set_from_bytes(&self, buffer: &[i8]) -> Result<()> {
        let buf_ptr = buffer.as_ptr();
        let buf_len = buffer.len() as u64;
        try_dpi!(
            externs::dpiLob_setFromBytes(self.inner, buf_ptr, buf_len),
            Ok(()),
            ErrorKind::Lob("dpiLob_setFromBytes".to_string())
        )
    }

    /// Trims the data in the LOB so that it only contains the specified amount of data.
    ///
    /// * `length` - the new size of the data in the LOB. For character LOBs this value is in
    /// characters; for binary LOBs this value is in bytes.
    pub fn trim(&self, length: u64) -> Result<()> {
        try_dpi!(
            externs::dpiLob_trim(self.inner, length),
            Ok(()),
            ErrorKind::Lob("dpiLob_trim".to_string())
        )
    }

    /// Write data to the LOB at the specified offset using the provided buffer as the source. If
    /// multiple calls to this function are planned, the LOB should first be opened using the
    /// function Lob::open_resource().
    ///
    /// * `buffer` - the buffer from which the data is written.
    /// * `offset` - the offset into the LOB data from which to start writing. The first position is
    /// 1. For character LOBs this represents the number of characters from the beginning of the
    /// LOB; for binary LOBS, this represents the number of bytes from the beginning of the LOB.
    pub fn write_bytes(&self, buffer: &[i8], offset: u64) -> Result<()> {
        let buf_ptr = buffer.as_ptr();
        let buf_len = buffer.len() as u64;

        try_dpi!(
            externs::dpiLob_writeBytes(self.inner, offset, buf_ptr, buf_len),
            Ok(()),
            ErrorKind::Lob("dpiLob_writeBytes".to_string())
        )
    }
}

impl From<*mut ODPILob> for Lob {
    fn from(inner: *mut ODPILob) -> Self {
        Self { inner: inner }
    }
}


impl Drop for Lob {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                externs::dpiLob_release(self.inner);
            }
        }
    }
}
