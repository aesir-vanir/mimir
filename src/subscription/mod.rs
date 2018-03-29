// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Subscription handles are used to represent subscriptions to events such as continuous query
//! notification and object change notification. They are created by calling the function
//! `Connection::new_subscription()` and are destroyed by calling the function
//! `Subscription::close()` or releasing the last reference by calling the function
//! `Subscription::release()`.
use error::{ErrorKind, Result};
use odpi::externs;
use odpi::opaque::ODPISubscr;
use statement::Statement;
use std::convert::TryFrom;
use std::ptr;
use util::ODPIStr;

/// ODPI-C Message Props wrapper.
#[derive(Clone)]
pub struct Subscription {
    /// The ODPI-C MsgProps pointer.
    inner: *mut ODPISubscr,
}

impl Subscription {
    #[doc(hidden)]
    pub fn inner(&self) -> *mut ODPISubscr {
        self.inner
    }

    /// Closes the subscription now, rather than when the last reference is released. This
    /// deregisters it so that notifications will no longer be sent.
    pub fn close(&self) -> Result<()> {
        try_dpi!(
            externs::dpiSubscr_close(self.inner),
            Ok(()),
            ErrorKind::Subscription("dpiSubscr_close".to_string())
        )
    }

    /// Prepares a statement for registration on the subscription. The statement is then registered
    /// by calling the function `Statement::execute()`. The reference to the statement that is
    /// returned should be released as soon as it is no longer needed.
    pub fn prepare_statement(&self, sql: &str) -> Result<Statement> {
        let mut stmt = ptr::null_mut();
        let sql_s: ODPIStr = TryFrom::try_from(sql)?;

        try_dpi!(
            externs::dpiSubscr_prepareStmt(self.inner, sql_s.ptr(), sql_s.len(), &mut stmt),
            Ok(stmt.into()),
            ErrorKind::Subscription("dpiSubscr_prepareStmt".to_string())
        )
    }
}

impl From<*mut ODPISubscr> for Subscription {
    fn from(inner: *mut ODPISubscr) -> Self {
        Self { inner }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                externs::dpiSubscr_release(self.inner);
            }
        }
    }
}
