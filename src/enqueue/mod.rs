// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Enqueue option handles are used to represent the options specified when enqueuing messages using
//! advanced queueing. They are created by calling the function `Connection::new_enq_options()` and
//! are destroyed by releasing the last reference by calling the function
//! `enqueue::Options::release()`.
use error::{ErrorKind, Result};
use odpi::{enums, externs};
use odpi::opaque::ODPIEnqOptions;
use std::convert::TryFrom;
use std::ptr;
use util::ODPIStr;

/// Enqueue option handles are used to represent the options specified when enqueuing messages.
#[derive(Clone)]
pub struct Options {
    /// The ODPI-C EnqOptions pointer.
    inner: *mut ODPIEnqOptions,
}

impl Options {
    #[doc(hidden)]
    pub fn inner(&self) -> *mut ODPIEnqOptions {
        self.inner
    }

    /// Adds a reference to the enqueue options. This is intended for situations where a reference
    /// to the enqueue options needs to be maintained independently of the reference returned when
    /// the handle was created.
    pub fn add_ref(&self) -> Result<()> {
        try_dpi!(
            externs::dpiEnqOptions_addRef(self.inner),
            Ok(()),
            ErrorKind::EnqOptions("dpiEnqOptions_addRef".to_string())
        )
    }

    /// Returns the transformation of the message to be enqueued. See function
    /// `enqueue::Options::set_transformation()` for more information.
    pub fn get_transformation(&self) -> Result<String> {
        let mut txn_ptr = ptr::null();
        let mut txn_len = 0;

        try_dpi!(
            externs::dpiEnqOptions_getTransformation(self.inner, &mut txn_ptr, &mut txn_len),
            {
                let transformation = if txn_ptr.is_null() {
                    "".to_string()
                } else {
                    let res_s = ODPIStr::new(txn_ptr, txn_len);
                    res_s.into()
                };
                Ok(transformation)
            },
            ErrorKind::EnqOptions("dpiEnqOptions_getTransformation".to_string())
        )
    }

    /// Returns whether the message being enqueued is part of the current transaction or constitutes
    /// a transaction on its own.
    pub fn get_visibility(&self) -> Result<enums::ODPIVisibility> {
        let mut enq_vis_ptr = enums::ODPIVisibility::Immediate;

        try_dpi!(
            externs::dpiEnqOptions_getVisibility(self.inner, &mut enq_vis_ptr),
            Ok(enq_vis_ptr),
            ErrorKind::EnqOptions("dpiEnqOptions_getMode".to_string())
        )
    }

    /// Releases a reference to the enqueue options. A count of the references to the enqueue
    /// options is maintained and when this count reaches zero, the memory associated with the
    /// options is freed.
    pub fn release(&self) -> Result<()> {
        try_dpi!(
            externs::dpiEnqOptions_release(self.inner),
            Ok(()),
            ErrorKind::EnqOptions("dpiEnqOptions_release".to_string())
        )
    }

    /// Sets the message delivery mode that is to be used when enqueuing messages.
    pub fn set_delivery_mode(&self, mode: enums::ODPIMessageDeliveryMode) -> Result<()> {
        try_dpi!(
            externs::dpiEnqOptions_setDeliveryMode(self.inner, mode),
            Ok(()),
            ErrorKind::EnqOptions("dpiEnqOptions_setDeliveryMode".to_string())
        )
    }

    /// Sets the transformation of the message to be enqueued. The transformation is applied after
    /// the message is enqueued but before it is returned to the application. It must be created
    /// using DBMS_TRANSFORM.
    pub fn set_transformation(&self, transformation: Option<&str>) -> Result<()> {
        let txn_s: ODPIStr = TryFrom::try_from(transformation)?;

        try_dpi!(
            externs::dpiEnqOptions_setTransformation(self.inner, txn_s.ptr(), txn_s.len()),
            Ok(()),
            ErrorKind::EnqOptions("dpiEnqOptions_setTransformation".to_string())
        )
    }

    /// Sets whether the message being enqueued is part of the current transaction or constitutes a
    /// transaction on its own.
    pub fn set_visibility(&self, visibility: enums::ODPIVisibility) -> Result<()> {
        try_dpi!(
            externs::dpiEnqOptions_setVisibility(self.inner, visibility),
            Ok(()),
            ErrorKind::EnqOptions("dpiEnqOptions_setVisibility".to_string())
        )
    }
}

impl From<*mut ODPIEnqOptions> for Options {
    fn from(inner: *mut ODPIEnqOptions) -> Self {
        Self { inner: inner }
    }
}
