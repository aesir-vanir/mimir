// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! [NOT IMPL]
//! This structure is used for passing query metadata from ODPI-C.
use data::TypeInfo;
use odpi::structs::ODPIQueryInfo;
use util::ODPIStr;

/// This structure is used for passing query metadata from ODPI-C. It is populated by the function
/// `Statement::get_query_info()`. All values remain valid as long as a reference is held to the
/// statement and the statement is not re-executed or closed.
pub struct Info {
    /// The ODPI-C query info struct.
    inner: ODPIQueryInfo,
}

impl Info {
    /// Create a new `Info` struct.
    pub fn new(inner: ODPIQueryInfo) -> Info {
        Info { inner: inner }
    }

    /// Get the `name` value.
    ///
    /// Specifies the name of the column which is being queried, as a string in the encoding used
    /// for CHAR data.
    pub fn name(&self) -> String {
        let name_s = ODPIStr::new(self.inner.name, self.inner.name_length);
        name_s.into()
    }

    /// Get the `scale` value.
    ///
    /// Specifies the scale of the column that is being queried. This value is only populated for
    /// numeric columns. For all other columns the value is zero.
    pub fn type_info(&self) -> TypeInfo {
        self.inner.type_info.into()
    }

    /// Get the `null_ok' value.
    ///
    /// Specifies if the column that is being queried may return null values or not.
    pub fn null_ok(&self) -> bool {
        self.inner.null_ok == 1
    }
}
