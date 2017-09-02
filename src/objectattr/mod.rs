// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Object attribute handles are used to represent the attributes of types such as those created by
//! the SQL command CREATE OR REPLACE TYPE. They are created by calling the function
//! `ODPIObjectType:g:et_attributes()` and are destroyed when the last reference is released by
//! calling the function `ODPIObjectAttr::release()`.
use error::{ErrorKind, Result};
use odpi::externs;
use odpi::opaque::ODPIObjectAttr;
use odpi::structs::ODPIObjectAttrInfo;

/// Object type handles are used to represent types such as those created by the SQL command CREATE
/// OR REPLACE TYPE.
#[derive(Clone, Debug)]
pub struct ObjectAttr {
    /// A pointer to the opaque `ODPIObjectAttr`.
    inner: *mut ODPIObjectAttr,
}

impl ObjectAttr {
    /// Get the pointer to the inner ODPI struct.
    #[doc(hidden)]
    pub fn inner(&self) -> *mut ODPIObjectAttr {
        self.inner
    }

    /// Returns information about the attribute.
    pub fn get_info(&self) -> Result<ODPIObjectAttrInfo> {
        let mut object_attr_info: ODPIObjectAttrInfo = Default::default();

        try_dpi!(
            externs::dpiObjectAttr_getInfo(self.inner, &mut object_attr_info),
            Ok(object_attr_info),
            ErrorKind::ObjectType("dpiObjectAttr_getInfo".to_string())
        )
    }

    /// Releases a reference to the attribute. A count of the references to the attribute is
    /// maintained and when this count reaches zero, the memory associated with the attribute is
    /// freed.
    pub fn release(&self) -> Result<()> {
        try_dpi!(
            externs::dpiObjectAttr_release(self.inner),
            Ok(()),
            ErrorKind::ObjectType("dpiObjectAttr_release".to_string())
        )
    }
}

impl From<*mut ODPIObjectAttr> for ObjectAttr {
    fn from(oot: *mut ODPIObjectAttr) -> Self {
        Self { inner: oot }
    }
}
